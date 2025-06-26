// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    sync::Arc,
    time::Duration,
};

use anyhow::Context as _;
use futures::FutureExt;
use livekit_api::{
    access_token::{AccessToken, VideoGrants},
    services::{ServiceError, TwirpError, TwirpErrorCode, room::RoomClient},
};
use livekit_protocol::TrackSource;
use opentalk_roomserver_common::settings::LiveKitSettings;
use opentalk_roomserver_signaling::{
    module_context::ModuleContext,
    signaling_module::{JoinInfo, PeerJoinInfoMap, SignalingModule, SignalingModuleInitData},
};
use opentalk_roomserver_types::{
    connection_id::ConnectionId, signaling::module_error::SignalingModuleError,
};
use opentalk_roomserver_types_livekit::{
    command::LiveKitCommand, error::LiveKitError, event::LiveKitEvent,
};
use opentalk_types_common::{
    modules::{ModuleId, module_id},
    rooms::RoomId,
};
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_livekit::{
    Credentials, MicrophoneRestrictionState, command::UnrestrictedParticipants, state::LiveKitState,
};
use tokio::sync::Mutex;

use crate::loopback::LiveKitLoopback;

pub mod loopback;

const LIVEKIT_MODULE_ID: ModuleId = module_id!("livekit");

const PARALLEL_UPDATES: usize = 25;
const ACCESS_TOKEN_TTL: Duration = Duration::from_secs(32);
const LIVEKIT_MEDIA_SOURCES: [TrackSource; 4] = [
    TrackSource::Camera,
    TrackSource::Microphone,
    TrackSource::ScreenShare,
    TrackSource::ScreenShareAudio,
];

pub struct LiveKitModule {
    settings: LiveKitSettings,

    /// The default screenshare permission. If the moderator didn't explicitly set a policy,
    /// this will be used to grant or deny screensharing privileges.
    ///
    /// `True` - all participants are allowed to screenshare
    /// `False` - only moderators are allowed to screenshare
    default_screenshare_permission: bool,

    /// LiveKit API client used to communicate with the LiveKit server
    livekit_client: Arc<RoomClient>,
    /// The record of all issued tokens for each participant.
    token_identities: BTreeMap<(ParticipantId, ConnectionId), BTreeSet<String>>,

    /// Activating the microphone can be restricted by the moderator. A subset of users might still be allowed to unmute.
    microphone_restrictions: MicrophoneRestrictionState,
    /// There must only be one background task updating the microphone restrictions at any given time.
    ongoing_microphone_restrictions: Arc<Mutex<()>>,
}

impl SignalingModule for LiveKitModule {
    const NAMESPACE: ModuleId = LIVEKIT_MODULE_ID;

    type Incoming = LiveKitCommand;

    type Outgoing = LiveKitEvent;

    type Loopback = Result<LiveKitLoopback, LiveKitError>;

    type JoinInfo = LiveKitState;

    type PeerJoinInfo = ();

    type Error = LiveKitError;

    fn init(init_data: SignalingModuleInitData) -> Option<Self> {
        let Some(livekit_settings) = &init_data.settings.livekit else {
            return None;
        };
        let default_screenshare_permission = init_data
            .settings
            .defaults
            .as_ref()
            .is_some_and(|d| !d.screen_share_requires_permission);

        let livekit_client = RoomClient::with_api_key(
            &livekit_settings.service_url,
            &livekit_settings.api_key,
            &livekit_settings.api_secret,
        );

        Some(Self {
            settings: livekit_settings.clone(),
            default_screenshare_permission,

            livekit_client: Arc::new(livekit_client),
            token_identities: BTreeMap::new(),

            microphone_restrictions: MicrophoneRestrictionState::Disabled,
            ongoing_microphone_restrictions: Mutex::new(()).into(),
        })
    }

    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<JoinInfo<Self>, SignalingModuleError<Self::Error>> {
        let livekit_client = Arc::clone(&self.livekit_client);
        let room_id = ctx.room_id;
        ctx.spawn(loopback::create_room(livekit_client, room_id));
        let token = self.create_new_access_token(ctx, participant_id, connection_id)?;
        Ok(JoinInfo {
            join_success: Some(LiveKitState {
                credentials: Credentials {
                    room: room_id.to_string(),
                    token,
                    public_url: self.settings.public_url.clone(),
                    service_url: None,
                },
                microphone_restriction_state: self.microphone_restrictions.clone(),
            }),
            peer: PeerJoinInfoMap::default(),
        })
    }

    fn on_participant_disconnected(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        // When a participant leaves, we revoke their access to livekit.
        // The identities will be removed from the set when the loopback task was successful.
        let Some(token_identities) = self
            .token_identities
            .get(&(participant_id, connection_id))
            .cloned()
        else {
            tracing::warn!("No livekit token identities found");
            return Ok(());
        };
        let livekit_client = self.livekit_client.clone();
        let room = ctx.room_id.to_string();

        ctx.spawn(loopback::revoke_token(
            livekit_client,
            participant_id,
            connection_id,
            room,
            token_identities,
        ));

        Ok(())
    }

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        connection_id: ConnectionId,
        content: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match content {
            LiveKitCommand::CreateNewAccessToken => {
                self.issue_access_token(ctx, sender, connection_id)
            }
            LiveKitCommand::ForceMute { participants } => {
                self.force_mute(ctx, sender, BTreeSet::from_iter(participants))
            }
            LiveKitCommand::GrantScreenSharePermission { participants } => {
                self.set_sceenshare_permissions(ctx, sender, participants, true)
            }
            LiveKitCommand::RevokeScreenSharePermission { participants } => {
                self.set_sceenshare_permissions(ctx, sender, participants, false)
            }
            LiveKitCommand::EnableMicrophoneRestrictions(unrestricted_participants) => self
                .update_microphone_restrictions(
                    ctx,
                    sender,
                    MicrophoneRestrictionState::Enabled {
                        unrestricted_participants: unrestricted_participants
                            .unrestricted_participants
                            .into_iter()
                            .collect(),
                    },
                ),
            LiveKitCommand::DisableMicrophoneRestrictions => self.update_microphone_restrictions(
                ctx,
                sender,
                MicrophoneRestrictionState::Disabled,
            ),
            LiveKitCommand::RequestPopoutStreamAccessToken => {
                self.issue_popout_stream_access_token(ctx, sender, connection_id)
            }
        }
    }

    fn on_loopback_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: Self::Loopback,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match event? {
            LiveKitLoopback::RoomCreated => Ok(()),

            LiveKitLoopback::ParticipantsMuted {
                sender,
                participants,
            } => self.notify_force_muted_participants(ctx, sender, participants),

            LiveKitLoopback::NoteRevokedTokens {
                token_identities,
                participant_id,
                connection_id,
            } => self.note_revoked_tokens(token_identities, participant_id, connection_id),

            LiveKitLoopback::ScreenSharePermissionsUpdated {
                sender,
                participants,
                grant,
            } => self.notify_screen_share_permission_update(ctx, sender, participants, grant),

            LiveKitLoopback::UpdatedMicrophoneRestrictions { sender: _, state } => {
                self.notify_microphone_restrictions_updated(ctx, state)
            }
        }
    }

    fn destroy(self, room_id: RoomId) {
        tokio::spawn(Self::cleanup_room(self.livekit_client, room_id));
    }
}

impl LiveKitModule {
    /// creates a new access token and sends it to the participant
    fn issue_access_token(
        &mut self,
        ctx: &mut ModuleContext<'_, LiveKitModule>,
        participant: ParticipantId,
        connection: ConnectionId,
    ) -> Result<(), SignalingModuleError<LiveKitError>> {
        tracing::debug!("Issue access token to {:?}", participant);
        let token = self.create_new_access_token(ctx, participant, connection)?;
        ctx.send_ws_message(
            [participant],
            LiveKitEvent::Credentials(Credentials {
                room: ctx.room_id.to_string(),
                token,
                public_url: self.settings.public_url.clone(),
                service_url: None,
            }),
        )?;
        Ok(())
    }

    fn create_new_access_token(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant: ParticipantId,
        connection: ConnectionId,
    ) -> Result<String, SignalingModuleError<<Self as SignalingModule>::Error>> {
        let mut available_sources = LIVEKIT_MEDIA_SOURCES.to_vec();
        if let MicrophoneRestrictionState::Enabled {
            unrestricted_participants,
        } = &self.microphone_restrictions
        {
            if !ctx.is_moderator(participant) && !unrestricted_participants.contains(&participant) {
                available_sources.retain(|s| s != &TrackSource::Microphone);
            }
        }

        if !self.default_screenshare_permission {
            available_sources
                .retain(|s| s != &TrackSource::ScreenShare && s != &TrackSource::ScreenShareAudio);
        };

        let can_publish_sources = available_sources
            .into_iter()
            .map(|s| TrackSource::as_str_name(&s).to_lowercase())
            .collect();

        let identity = build_livekit_participant_id(participant, connection);

        let hidden = !ctx
            .participant_state(participant)
            .with_context(|| format!("Participant '{}' has no state", participant))?
            .is_visible();

        let access_token =
            AccessToken::with_api_key(&self.settings.api_key, &self.settings.api_secret)
                .with_name(&identity)
                .with_identity(&identity)
                .with_grants(VideoGrants {
                    room_create: true,
                    room_list: false,
                    room_record: false,
                    room_admin: false,
                    room_join: true,
                    room: ctx.room_id.to_string(),
                    can_publish: true,
                    can_subscribe: true,
                    can_publish_data: false,
                    can_publish_sources,
                    can_update_own_metadata: false,
                    ingress_admin: false,
                    hidden,
                    recorder: false,
                    destination_room: String::new(),
                })
                .with_ttl(ACCESS_TOKEN_TTL)
                .to_jwt()
                .context("Failed to create LiveKit access-token")?;
        self.token_identities
            .entry((participant, connection))
            .or_default()
            .insert(identity);

        Ok(access_token)
    }

    fn force_mute(
        &self,
        ctx: &mut ModuleContext<'_, LiveKitModule>,
        sender: ParticipantId,
        participants: BTreeSet<ParticipantId>,
    ) -> Result<(), SignalingModuleError<<Self as SignalingModule>::Error>> {
        if !ctx.is_moderator(sender) {
            tracing::debug!(
                "Participant has insufficient permission to force mute participants: {sender}"
            );
            return Err(LiveKitError::InsufficientPermissions.into());
        }

        // check that we know all participants and query their connection ids
        let known_participants: BTreeSet<_> = ctx
            .participants
            .connected()
            .ids()
            .filter(|p| participants.contains(p))
            .collect();
        let unknown_participants: BTreeSet<_> = participants
            .difference(&known_participants)
            .copied()
            .collect();

        let mut connections = ctx.participants.connections();
        connections.retain(|p, _| known_participants.contains(p));
        Self::notify_unknown_participants(unknown_participants, ctx, sender)?;

        let room = ctx.room_id.to_string();
        let livekit_client: Arc<RoomClient> = Arc::clone(&self.livekit_client);

        tracing::debug!("spawn background task to force mute participants");
        ctx.spawn(loopback::force_mute_participants(
            livekit_client,
            sender,
            connections,
            room,
        ));
        Ok(())
    }

    fn notify_force_muted_participants(
        &self,
        ctx: &mut ModuleContext<'_, LiveKitModule>,
        sender: ParticipantId,
        participants: BTreeSet<ParticipantId>,
    ) -> Result<(), SignalingModuleError<<Self as SignalingModule>::Error>> {
        tracing::debug!("Participants have been force muted");
        ctx.send_ws_message(participants, LiveKitEvent::ForceMuted { moderator: sender })?;
        Ok(())
    }

    fn note_revoked_tokens(
        &mut self,
        revoked_token_identities: BTreeSet<String>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<LiveKitError>> {
        let entry = self.token_identities.entry((participant_id, connection_id));

        if let Entry::Occupied(mut occupied) = entry {
            occupied
                .get_mut()
                .retain(|item| !revoked_token_identities.contains(item));
            if occupied.get().is_empty() {
                occupied.remove();
            }
        }
        Ok(())
    }

    fn set_sceenshare_permissions(
        &mut self,
        ctx: &mut ModuleContext<'_, LiveKitModule>,
        sender: ParticipantId,
        participants: BTreeSet<ParticipantId>,
        grant: bool,
    ) -> Result<(), SignalingModuleError<LiveKitError>> {
        if !ctx.is_moderator(sender) {
            tracing::debug!(
                "Participant has insufficient permission to grant screen sharing rights: {sender}"
            );
            return Err(LiveKitError::InsufficientPermissions.into());
        }
        let room = ctx.room_id.to_string();

        // check that we know all participants and query their connection ids
        let known_participants: BTreeSet<_> = ctx
            .participants
            .connected()
            .ids()
            .filter(|p| participants.contains(p))
            .collect();
        let unknown_participants: BTreeSet<_> = participants
            .difference(&known_participants)
            .copied()
            .collect();

        let mut connections = ctx.participants.connections();
        connections.retain(|p, _| known_participants.contains(p));
        Self::notify_unknown_participants(unknown_participants, ctx, sender)?;

        ctx.spawn(loopback::set_sceenshare_permissions(
            Arc::clone(&self.livekit_client),
            room,
            sender,
            connections,
            grant,
        ));

        Ok(())
    }

    fn notify_screen_share_permission_update(
        &self,
        ctx: &mut ModuleContext<'_, LiveKitModule>,
        sender: ParticipantId,
        participants: BTreeSet<ParticipantId>,
        grant: bool,
    ) -> Result<(), SignalingModuleError<LiveKitError>> {
        ctx.send_ws_message(
            [sender],
            LiveKitEvent::ScreenSharePermissionsUpdated {
                grant,
                participants: participants.into_iter().collect(),
            },
        )?;
        Ok(())
    }

    fn issue_popout_stream_access_token(
        &mut self,
        ctx: &mut ModuleContext<'_, LiveKitModule>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<LiveKitError>> {
        let identity = format!(
            "{}-popout{}",
            build_livekit_participant_id(participant_id, connection_id),
            self.token_identities
                .get(&(participant_id, connection_id))
                .map(|s| s.len())
                .unwrap_or_default()
        );

        let token = AccessToken::with_api_key(&self.settings.api_key, &self.settings.api_secret)
            .with_name(&identity)
            .with_identity(&identity)
            .with_grants(VideoGrants {
                room_create: false,
                room_list: false,
                room_record: false,
                room_admin: false,
                room_join: true,
                room: ctx.room_id.to_string(),
                can_publish: false,
                can_subscribe: true,
                can_publish_data: false,
                can_publish_sources: vec![],
                can_update_own_metadata: false,
                ingress_admin: false,
                hidden: true,
                recorder: false,
                destination_room: String::new(),
            })
            .with_ttl(ACCESS_TOKEN_TTL)
            .to_jwt()
            .map_err(|err| {
                tracing::error!("failed to create popout stream access token: {}", err);
                LiveKitError::LivekitUnavailable
            })?;

        self.token_identities
            .entry((participant_id, connection_id))
            .or_default()
            .insert(identity);

        ctx.send_ws_message(
            [participant_id],
            LiveKitEvent::PopoutStreamAccessToken { token },
        )?;

        Ok(())
    }

    fn update_microphone_restrictions(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        new_state: MicrophoneRestrictionState,
    ) -> Result<(), SignalingModuleError<LiveKitError>> {
        if !ctx.is_moderator(sender) {
            tracing::debug!(
                "Participant has insufficient permission to update microphone restrictions: {sender}"
            );
            return Err(LiveKitError::InsufficientPermissions.into());
        }
        let local_lock = Arc::clone(&self.ongoing_microphone_restrictions);
        let Ok(guard) = local_lock.try_lock_owned() else {
            tracing::debug!(
                "Received microphone restriction request during ongoing restriction update"
            );
            return Err(LiveKitError::ConflictingTask.into());
        };

        let room = ctx.room_id.to_string();
        let livekit_client = Arc::clone(&self.livekit_client);
        let connections = ctx.participants.connections();

        // update the state now so that the rule is already applied to new participants.
        self.microphone_restrictions = new_state.clone();

        ctx.spawn({
            loopback::update_restricted_microphones(
                livekit_client,
                room,
                sender,
                new_state,
                connections,
            )
            .map(move |res| {
                drop(guard);
                res
            })
        });

        Ok(())
    }

    fn notify_microphone_restrictions_updated(
        &mut self,
        ctx: &mut ModuleContext<'_, LiveKitModule>,
        _state: MicrophoneRestrictionState,
    ) -> Result<(), SignalingModuleError<LiveKitError>> {
        match &self.microphone_restrictions {
            MicrophoneRestrictionState::Disabled => {
                ctx.send_ws_message(
                    ctx.participants.connected().iter().map(|(id, _)| *id),
                    LiveKitEvent::MicrophoneRestrictionsDisabled,
                )?;
            }
            MicrophoneRestrictionState::Enabled {
                unrestricted_participants,
            } => {
                ctx.send_ws_message(
                    ctx.participants.connected().iter().map(|(id, _)| *id),
                    LiveKitEvent::MicrophoneRestrictionsEnabled(UnrestrictedParticipants {
                        unrestricted_participants: unrestricted_participants
                            .iter()
                            .copied()
                            .collect(),
                    }),
                )?;
            }
        }

        Ok(())
    }

    async fn cleanup_room(livekit_client: Arc<RoomClient>, room_id: RoomId) {
        match livekit_client.delete_room(&room_id.to_string()).await {
            Ok(_) => {
                tracing::debug!("Destroyed livekit room with the id {}", room_id);
            }
            Err(ServiceError::Twirp(TwirpError::Twirp(code)))
                if code.code == TwirpErrorCode::NOT_FOUND =>
            {
                tracing::debug!("Livekit room with the id {} was already destroyed", room_id);
            }
            Err(e) => {
                tracing::error!("Failed to destroy livekit room {}: {}", room_id, e);
            }
        }
    }

    fn notify_unknown_participants(
        unknown_participants: BTreeSet<ParticipantId>,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
    ) -> Result<(), SignalingModuleError<LiveKitError>> {
        if !unknown_participants.is_empty() {
            ctx.send_ws_message(
                [sender],
                LiveKitError::UnknownParticipant {
                    participant: unknown_participants,
                }
                .into(),
            )?;
            Ok(())
        } else {
            Ok(())
        }
    }
}

fn build_livekit_participant_id(participant: ParticipantId, connection: ConnectionId) -> String {
    format!("{}:{}", participant, connection)
}
