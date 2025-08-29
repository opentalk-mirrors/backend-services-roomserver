// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::Arc,
    time::Duration,
};

use anyhow::Context;
use livekit_api::{
    access_token::{AccessToken, VideoGrants},
    services::room::RoomClient,
};
use livekit_protocol::TrackSource;
use opentalk_roomserver_signaling::{
    module_context::ModuleContext,
    signaling_module::{ModuleJoinData, NoOp, SignalingModule, SignalingModuleInitData},
};
use opentalk_roomserver_types::{
    connection_id::ConnectionId, signaling::module_error::SignalingModuleError,
};
use opentalk_roomserver_types_livekit::LiveKitSettings;
use opentalk_roomserver_types_subroom_audio::{
    SUBROOM_AUDIO_MODULE_ID, WhisperId,
    command::{ParticipantTargets, SubroomAudioCommand},
    event::{
        ParticipantsInvited, SubroomAudioError, SubroomAudioEvent, WhisperAccepted, WhisperInvite,
        WhisperParticipantInfo, WhisperToken,
    },
    state::{WhisperGroup, WhisperState},
};
use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::ParticipantId;

use crate::loopback::SubroomAudioLoopback;

pub mod loopback;

const ACCESS_TOKEN_TTL: Duration = Duration::from_secs(32);

pub struct SubroomAudioModule {
    settings: Arc<LiveKitSettings>,
    livekit_client: Arc<RoomClient>,
    whisper_rooms: HashMap<WhisperId, WhisperGroup>,
}

impl SignalingModule for SubroomAudioModule {
    const NAMESPACE: ModuleId = SUBROOM_AUDIO_MODULE_ID;

    type Incoming = SubroomAudioCommand;

    type Outgoing = SubroomAudioEvent;

    type Internal = NoOp;

    type Loopback = Result<SubroomAudioLoopback, SubroomAudioError>;

    type JoinInfo = ();

    type PeerJoinInfo = ();

    type Error = SubroomAudioError;

    fn init(init_data: SignalingModuleInitData) -> Option<Self> {
        let livekit_settings = (init_data
            .room_parameters
            .module_data
            .get::<LiveKitSettings>()
            .ok()?)?;

        let livekit_client = RoomClient::with_api_key(
            &livekit_settings.service_url,
            &livekit_settings.api_key,
            &livekit_settings.api_secret,
        );

        Some(Self {
            settings: Arc::new(livekit_settings.clone()),

            livekit_client: Arc::new(livekit_client),

            whisper_rooms: HashMap::new(),
        })
    }

    fn on_participant_joined(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
        _participant_id: ParticipantId,
        _connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Self::Error>> {
        Ok(ModuleJoinData::default())
    }

    fn on_participant_disconnected(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        let result = self.leave_all_whisper_groups(ctx, participant_id);

        if result.is_err() {
            tracing::debug!(
                "Failed to remove participant {participant_id} from all whisper groups: {:?}",
                result.err()
            );
        }

        Ok(())
    }

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        _connection_id: ConnectionId,
        payload: Self::Incoming,
    ) -> Result<(), SignalingModuleError<SubroomAudioError>> {
        match payload {
            SubroomAudioCommand::LeaveWhisperGroup { whisper_id } => {
                self.leave_whisper_group(ctx, sender, whisper_id)
            }
            SubroomAudioCommand::CreateWhisperGroup { participant_ids } => {
                self.create_whisper_group(ctx, sender, participant_ids)
            }
            SubroomAudioCommand::InviteToWhisperGroup(participant_targets) => {
                self.invite_to_whisper_group(ctx, sender, participant_targets)
            }
            SubroomAudioCommand::KickWhisperParticipants(participant_targets) => {
                self.kick_whisper_participants(ctx, sender, participant_targets)
            }
            SubroomAudioCommand::AcceptWhisperInvite { whisper_id } => {
                self.accept_whisper_invite(ctx, sender, whisper_id)
            }
            SubroomAudioCommand::DeclineWhisperInvite { whisper_id } => {
                self.decline_whisper_invite(ctx, sender, whisper_id)
            }
        }
    }
}

impl SubroomAudioModule {
    fn get_whisper_group(
        &mut self,
        sender: ParticipantId,
        whisper_id: WhisperId,
    ) -> Result<&mut WhisperGroup, SignalingModuleError<SubroomAudioError>> {
        let Some(whisper_group) = self.whisper_rooms.get_mut(&whisper_id) else {
            return Err(SubroomAudioError::InvalidWhisperId.into());
        };

        if !whisper_group
            .participants
            .iter()
            .any(|(participant_id, _)| participant_id == &sender)
        {
            return Err(SubroomAudioError::NotInvited.into());
        }

        Ok(whisper_group)
    }

    fn create_whisper_group(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        participant_ids: BTreeSet<ParticipantId>,
    ) -> Result<(), SignalingModuleError<SubroomAudioError>> {
        if participant_ids.is_empty() {
            return Err(SubroomAudioError::EmptyParticipantList.into());
        }

        self.participant_targets_valid(ctx, &participant_ids)?;

        let whisper_id = WhisperId::generate();

        let token = self.create_room_and_access_token(ctx, sender, whisper_id)?;

        let mut whisper_participants = participant_ids
            .into_iter()
            .map(|participant_id| (participant_id, WhisperState::default()))
            .collect::<BTreeMap<_, _>>();

        whisper_participants.insert(sender, WhisperState::Creator);

        let whisper_group = WhisperGroup {
            whisper_id,
            participants: whisper_participants.clone(),
        };

        self.whisper_rooms.insert(whisper_id, whisper_group.clone());

        ctx.send_ws_message(
            whisper_participants
                .keys()
                .filter(|&&p| p != sender)
                .cloned(),
            SubroomAudioEvent::WhisperInvite(WhisperInvite {
                issuer: sender,
                group: whisper_group.clone().into(),
            }),
        )?;

        ctx.send_ws_message(
            [sender],
            SubroomAudioEvent::WhisperGroupCreated {
                token,
                group: whisper_group.into(),
            },
        )?;

        Ok(())
    }

    fn accept_whisper_invite(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        whisper_id: WhisperId,
    ) -> Result<(), SignalingModuleError<SubroomAudioError>> {
        let token = self.create_access_token(sender, whisper_id)?;
        let whisper_group = self.get_whisper_group(sender, whisper_id)?;

        if whisper_group.has_accepted(&sender) {
            return Err(SubroomAudioError::AlreadyAccepted.into());
        }

        if let Some(state) = whisper_group.participants.get_mut(&sender) {
            *state = WhisperState::Accepted;
        } else {
            return Err(SubroomAudioError::NotInvited.into());
        }

        ctx.send_ws_message(
            [sender],
            SubroomAudioEvent::WhisperToken(WhisperToken { whisper_id, token }),
        )?;

        ctx.send_ws_message(
            whisper_group
                .participants
                .keys()
                .filter(|&&p| p != sender)
                .cloned(),
            SubroomAudioEvent::WhisperInviteAccepted(WhisperAccepted {
                whisper_id,
                participant_id: sender,
            }),
        )?;

        Ok(())
    }

    fn decline_whisper_invite(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        whisper_id: WhisperId,
    ) -> Result<(), SignalingModuleError<SubroomAudioError>> {
        let whisper_group = self.get_whisper_group(sender, whisper_id)?;

        whisper_group.participants.remove(&sender);

        ctx.send_ws_message(
            whisper_group
                .participants
                .keys()
                .filter(|&&p| p != sender)
                .cloned(),
            SubroomAudioEvent::WhisperInviteDeclined(WhisperParticipantInfo {
                whisper_id,
                participant_id: sender,
            }),
        )?;

        Ok(())
    }

    fn leave_whisper_group(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        whisper_id: WhisperId,
    ) -> Result<(), SignalingModuleError<SubroomAudioError>> {
        let livekit_client = Arc::clone(&self.livekit_client);
        let whisper_group = self.get_whisper_group(sender, whisper_id)?;
        let has_token = whisper_group.has_accepted(&sender);

        if has_token {
            ctx.spawn(loopback::remove_participant(
                livekit_client,
                whisper_id.to_string(),
                sender.to_string(),
            ));
        }

        whisper_group.participants.remove(&sender);

        if whisper_group.participants.is_empty() {
            ctx.spawn(loopback::destroy_room(
                Arc::clone(&self.livekit_client),
                whisper_id.to_string(),
            ));

            self.whisper_rooms.remove(&whisper_id);
        } else {
            ctx.send_ws_message(
                whisper_group
                    .participants
                    .keys()
                    .filter(|&&p| p != sender)
                    .cloned(),
                SubroomAudioEvent::LeftWhisperGroup(WhisperParticipantInfo {
                    whisper_id,
                    participant_id: sender,
                }),
            )?;
        }

        Ok(())
    }

    fn leave_all_whisper_groups(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
    ) -> Result<(), SignalingModuleError<SubroomAudioError>> {
        for whisper_id in self.whisper_rooms.keys().cloned().collect::<Vec<_>>() {
            let Ok(whisper_group) = self.get_whisper_group(sender, whisper_id) else {
                continue;
            };

            if whisper_group
                .participants
                .iter()
                .any(|(partipand_id, _)| partipand_id == &sender)
            {
                self.leave_whisper_group(ctx, sender, whisper_id)?;
            }
        }

        Ok(())
    }

    fn invite_to_whisper_group(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        participant_targets: ParticipantTargets,
    ) -> Result<(), SignalingModuleError<SubroomAudioError>> {
        self.participant_targets_valid(ctx, &participant_targets.participant_ids)?;
        let whisper_group = self.get_whisper_group(sender, participant_targets.whisper_id)?;

        if !is_group_creator(&sender, whisper_group) {
            return Err(SubroomAudioError::InsufficientPermissions.into());
        }

        let new_participants = participant_targets
            .participant_ids
            .iter()
            .filter(|&&participant_id| {
                let invitable_participant =
                    !whisper_group.contains(&participant_id) && participant_id != sender;

                if !invitable_participant {
                    tracing::debug!("Skip to invite participant {participant_id}");
                }

                invitable_participant
            })
            .map(|participant_id| (*participant_id, WhisperState::Invited))
            .collect::<BTreeMap<_, _>>();

        if new_participants.is_empty() {
            return Ok(());
        }

        let original_participant_ids: Vec<_> = whisper_group
            .participants
            .keys()
            .copied()
            .filter(|&p| p != sender)
            .collect();
        let new_participant_ids: Vec<_> = new_participants.keys().copied().collect();

        for participant_id in new_participant_ids.clone() {
            whisper_group
                .participants
                .insert(participant_id, WhisperState::Invited);
        }

        ctx.send_ws_message(
            new_participant_ids.clone(),
            SubroomAudioEvent::WhisperInvite(WhisperInvite {
                issuer: sender,
                group: whisper_group.clone().into(),
            }),
        )?;

        let participants_invited_event = ParticipantsInvited {
            whisper_id: participant_targets.whisper_id,
            participant_ids: new_participant_ids,
        };

        // This is only send to the original whisper group participants
        ctx.send_ws_message(
            original_participant_ids,
            SubroomAudioEvent::ParticipantsInvited(participants_invited_event),
        )?;

        Ok(())
    }

    fn kick_whisper_participants(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        participant_targets: ParticipantTargets,
    ) -> Result<(), SignalingModuleError<SubroomAudioError>> {
        self.participant_targets_valid(ctx, &participant_targets.participant_ids)?;

        let whisper_group = self.get_whisper_group(sender, participant_targets.whisper_id)?;
        let participants_to_kick: BTreeSet<_> = participant_targets
            .participant_ids
            .into_iter()
            .filter(|&p| p != sender)
            .collect();

        if !is_group_creator(&sender, whisper_group) {
            return Err(SubroomAudioError::InsufficientPermissions.into());
        }

        whisper_group
            .participants
            .retain(|participant_id, _| !participants_to_kick.contains(participant_id));

        ctx.send_ws_message(
            participants_to_kick,
            SubroomAudioEvent::Kicked {
                whisper_id: participant_targets.whisper_id,
            },
        )?;

        Ok(())
    }

    fn participant_targets_valid(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        participants: &BTreeSet<ParticipantId>,
    ) -> Result<(), SignalingModuleError<SubroomAudioError>> {
        if participants.is_empty() {
            return Err(SubroomAudioError::EmptyParticipantList.into());
        }

        let room_participants = &ctx.participants.connected().ids().collect();
        let invalid_participants: Vec<_> = participants
            .difference(room_participants)
            .copied()
            .collect();

        if !invalid_participants.is_empty() {
            return Err(SubroomAudioError::InvalidParticipantTargets {
                participant_ids: invalid_participants,
            }
            .into());
        }

        Ok(())
    }

    fn create_room_and_access_token(
        &self,
        ctx: &ModuleContext<'_, Self>,
        sender: ParticipantId,
        whisper_id: WhisperId,
    ) -> Result<String, SignalingModuleError<SubroomAudioError>> {
        ctx.spawn(loopback::create_room(
            Arc::clone(&self.livekit_client),
            whisper_id.to_string(),
        ));

        let token = self.create_access_token(sender, whisper_id)?;

        Ok(token)
    }

    fn create_access_token(
        &self,
        sender: ParticipantId,
        whisper_id: WhisperId,
    ) -> Result<String, SignalingModuleError<SubroomAudioError>> {
        let identity = &sender.to_string();

        let access_token =
            AccessToken::with_api_key(&self.settings.api_key, &self.settings.api_secret)
                .with_name(identity)
                .with_identity(identity)
                .with_grants(VideoGrants {
                    room_create: false,
                    room_list: false,
                    room_record: false,
                    room_admin: false,
                    room_join: true,
                    room: whisper_id.to_string(),
                    destination_room: String::new(),
                    can_publish: true,
                    can_subscribe: true,
                    can_publish_data: false,
                    can_publish_sources: vec![TrackSource::Microphone.as_str_name().to_lowercase()],
                    can_update_own_metadata: false,
                    ingress_admin: false,
                    hidden: false,
                    recorder: false,
                })
                .with_ttl(ACCESS_TOKEN_TTL)
                .to_jwt()
                .context("Failed to create livekit access-token")?;

        Ok(access_token)
    }
}

fn is_group_creator(sender: &ParticipantId, whisper_group: &WhisperGroup) -> bool {
    matches!(
        whisper_group.participants.get(sender),
        Some(WhisperState::Creator)
    )
}
