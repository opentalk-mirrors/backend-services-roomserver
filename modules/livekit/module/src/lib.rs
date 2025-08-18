// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    iter::repeat,
    sync::Arc,
    time::Duration,
};

use futures::{StreamExt as _, stream};
use livekit_api::services::room::RoomClient;
use livekit_protocol::TrackSource;
use opentalk_roomserver_signaling::{
    module_context::ModuleContext,
    signaling_module::{JoinInfo, NoOp, SignalingModule, SignalingModuleInitData},
};
use opentalk_roomserver_types::{
    breakout::BreakoutRoom, connection_id::ConnectionId, room_kind::RoomKind,
    signaling::module_error::SignalingModuleError,
};
use opentalk_roomserver_types_livekit::{
    LiveKitCommand, LiveKitError, LiveKitEvent, LiveKitSettings, LiveKitState,
    MicrophoneRestrictionState,
};
use opentalk_types_common::{
    modules::{ModuleId, module_id},
    rooms::RoomId,
};
use opentalk_types_signaling::ParticipantId;
use tracing::{Instrument, Span};

use crate::{loopback::LiveKitLoopback, room::LiveKitSubroom};

pub mod loopback;
mod room;

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
    settings: Arc<LiveKitSettings>,

    /// The default screenshare permission. If the moderator didn't explicitly set a policy,
    /// this will be used to grant or deny screensharing privileges.
    ///
    /// `True` - all participants are allowed to screenshare
    /// `False` - only moderators are allowed to screenshare
    default_screenshare_permission: bool,

    /// LiveKit API client used to communicate with the LiveKit server
    livekit_client: Arc<RoomClient>,

    rooms: HashMap<RoomKind, LiveKitSubroom>,
}

impl SignalingModule for LiveKitModule {
    const NAMESPACE: ModuleId = LIVEKIT_MODULE_ID;

    type Incoming = LiveKitCommand;

    type Outgoing = LiveKitEvent;

    type Internal = NoOp;

    type Loopback = Result<LiveKitLoopback, LiveKitError>;

    type JoinInfo = LiveKitState;

    type PeerJoinInfo = ();

    type Error = LiveKitError;

    fn init(init_data: SignalingModuleInitData) -> Option<Self> {
        let livekit_settings = (init_data
            .room_parameters
            .module_data
            .get::<LiveKitSettings>()
            .ok()?)?;

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
            settings: Arc::new(livekit_settings.clone()),
            default_screenshare_permission,

            livekit_client: Arc::new(livekit_client),

            rooms: HashMap::new(),
        })
    }

    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<JoinInfo<Self>, SignalingModuleError<Self::Error>> {
        let room = self.rooms.entry(ctx.room).or_insert_with(|| {
            LiveKitSubroom::new(
                ctx,
                self.default_screenshare_permission,
                Arc::clone(&self.settings),
                Arc::clone(&self.livekit_client),
                ctx.room_id,
                ctx.room,
            )
        });

        room.join_info(ctx, participant_id, connection_id)
    }

    fn on_participant_disconnected(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        let Some(room) = self.rooms.get_mut(&ctx.room) else {
            return Err(anyhow::anyhow!("Unknown room").into());
        };
        room.start_revoke_participant_access(ctx, participant_id, connection_id);
        Ok(())
    }

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        connection_id: ConnectionId,
        payload: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match payload {
            LiveKitCommand::CreateNewAccessToken => {
                self.issue_access_token(ctx, sender, connection_id)
            }
            LiveKitCommand::ForceMute { participants } => {
                self.force_mute(ctx, sender, participants)
            }
            LiveKitCommand::GrantScreenSharePermission { participants } => {
                self.set_screenshare_permissions(ctx, sender, participants, true)
            }
            LiveKitCommand::RevokeScreenSharePermission { participants } => {
                self.set_screenshare_permissions(ctx, sender, participants, false)
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
            LiveKitLoopback::RoomRemoved => Ok(()),

            LiveKitLoopback::ParticipantsMuted {
                sender,
                participants,
            } => self.notify_force_muted_participants(ctx, sender, participants),
            LiveKitLoopback::NoteRevokedTokens {
                token_identities,
                participant_id,
                connection_id,
            } => self.note_revoked_tokens(ctx, token_identities, participant_id, connection_id),
            LiveKitLoopback::ScreenSharePermissionsUpdated {
                sender,
                participants,
                grant,
            } => self.notify_screen_share_permission_update(ctx, sender, participants, grant),
            LiveKitLoopback::UpdatedMicrophoneRestrictions { .. } => {
                self.notify_microphone_restrictions_updated(ctx)
            }
        }
    }

    fn destroy(self, _room_id: RoomId) {
        let span = Span::current();
        let rooms = self.rooms.into_values().zip(repeat(span));
        let futures = stream::iter(rooms)
            .map(|(r, span)| r.cleanup_room().instrument(span))
            .buffer_unordered(PARALLEL_UPDATES)
            .collect::<Vec<()>>();
        tokio::spawn(futures);
    }

    fn on_breakout_start(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        rooms: &[BreakoutRoom],
        _duration: Option<Duration>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        for room in rooms {
            self.rooms
                .entry(RoomKind::Breakout(room.id))
                .or_insert_with(|| {
                    let room_kind = RoomKind::Breakout(room.id);
                    tracing::debug!("create room: {:?}", room_kind);
                    LiveKitSubroom::new(
                        ctx,
                        self.default_screenshare_permission,
                        self.settings.clone(),
                        Arc::clone(&self.livekit_client),
                        ctx.room_id,
                        room_kind,
                    )
                });
        }
        Ok(())
    }

    fn on_breakout_switch(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        old_room: RoomKind,
        new_room: RoomKind,
    ) -> Result<BTreeMap<ConnectionId, Option<Self::JoinInfo>>, SignalingModuleError<Self::Error>>
    {
        let connections = ctx.participants.connections();
        let connections = connections.get(&participant_id).ok_or_else(|| {
            anyhow::anyhow!("Unknown participant can't switch breakout rooms {participant_id}")
        })?;

        let Some(room) = self.rooms.get_mut(&old_room) else {
            return Err(anyhow::anyhow!(
                "Source room not found while switching breakout rooms ({old_room:?})"
            )
            .into());
        };
        for connection_id in connections {
            room.start_revoke_participant_access(ctx, participant_id, *connection_id);
        }

        let Some(room) = self.rooms.get_mut(&new_room) else {
            return Err(anyhow::anyhow!(
                "Destination room not found while switching breakout rooms ({new_room:?})"
            )
            .into());
        };
        let mut join_infos = BTreeMap::new();
        for &connection_id in connections {
            let join_info = room
                .join_info(ctx, participant_id, connection_id)?
                .join_success;
            join_infos.insert(connection_id, join_info);
        }
        Ok(join_infos)
    }

    fn on_breakout_closed(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        let breakout_rooms: HashMap<_, _> = self
            .rooms
            .extract_if(|kind, _| *kind != RoomKind::Main)
            .collect();

        ctx.spawn(async {
            stream::iter(breakout_rooms.into_iter().map(|(id, r)| async move {
                r.cleanup_room().await;
                tracing::debug!("LiveKitRoom removed: {id:?}");
            }))
            .buffer_unordered(PARALLEL_UPDATES)
            .collect::<Vec<()>>()
            .await;

            Ok(LiveKitLoopback::RoomRemoved)
        });
        Ok(())
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
        let Some(room) = self.rooms.get_mut(&ctx.room) else {
            return Err(anyhow::anyhow!("Unknown room").into());
        };
        tracing::debug!("Issue access token to {participant:?}");
        let credentials = room.create_new_access_token(ctx, participant, connection)?;
        ctx.send_ws_message([participant], LiveKitEvent::Credentials(credentials))?;
        Ok(())
    }

    fn force_mute(
        &self,
        ctx: &mut ModuleContext<'_, LiveKitModule>,
        sender: ParticipantId,
        participants: BTreeSet<ParticipantId>,
    ) -> Result<(), SignalingModuleError<<Self as SignalingModule>::Error>> {
        let Some(room) = self.rooms.get(&ctx.room) else {
            return Err(anyhow::anyhow!("Unknown room").into());
        };
        room.force_mute(ctx, sender, participants)?;

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
        ctx: &mut ModuleContext<'_, LiveKitModule>,
        revoked_token_identities: BTreeSet<String>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<LiveKitError>> {
        let Some(room) = self.rooms.get_mut(&ctx.room) else {
            return Err(anyhow::anyhow!("Unknown room").into());
        };
        room.note_revoked_tokens(revoked_token_identities, participant_id, connection_id)
    }

    fn set_screenshare_permissions(
        &mut self,
        ctx: &mut ModuleContext<'_, LiveKitModule>,
        sender: ParticipantId,
        participants: BTreeSet<ParticipantId>,
        grant: bool,
    ) -> Result<(), SignalingModuleError<LiveKitError>> {
        let Some(room) = self.rooms.get_mut(&ctx.room) else {
            return Err(anyhow::anyhow!("Unknown room").into());
        };
        room.set_screenshare_permissions(ctx, sender, participants, grant)
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
                participants,
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
        let Some(room) = self.rooms.get_mut(&ctx.room) else {
            return Err(anyhow::anyhow!("Unknown room").into());
        };
        room.issue_popout_stream_access_token(ctx, participant_id, connection_id)
    }

    fn update_microphone_restrictions(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        new_state: MicrophoneRestrictionState,
    ) -> Result<(), SignalingModuleError<LiveKitError>> {
        let Some(room) = self.rooms.get_mut(&ctx.room) else {
            return Err(anyhow::anyhow!("Unknown room").into());
        };
        room.update_microphone_restrictions(ctx, sender, new_state)
    }

    fn notify_microphone_restrictions_updated(
        &mut self,
        ctx: &mut ModuleContext<'_, LiveKitModule>,
    ) -> Result<(), SignalingModuleError<LiveKitError>> {
        let Some(room) = self.rooms.get_mut(&ctx.room) else {
            return Err(anyhow::anyhow!("Unknown room").into());
        };
        room.notify_microphone_restrictions_updated(ctx)
    }
}

fn build_livekit_participant_id(participant: ParticipantId, connection: ConnectionId) -> String {
    format!("{participant}:{connection}")
}
