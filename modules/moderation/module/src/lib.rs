// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{collections::HashMap, mem};

use opentalk_roomserver_module_livekit::LiveKitModule;
use opentalk_roomserver_signaling::{
    module_context::{ChannelDroppedError, ModuleContext},
    participant_state::ParticipantState,
    signaling_module::{ModuleJoinData, NoOp, SignalingModule, SignalingModuleInitData},
};
use opentalk_roomserver_types::{
    client_parameters::ClientKind,
    connection_id::ConnectionId,
    signaling::module_error::{FatalError, SignalingModuleError},
};
use opentalk_roomserver_types_livekit::{
    LiveKitInternal, MicrophoneRestrictionError, MicrophoneRestrictionState,
};
use opentalk_roomserver_types_moderation::{
    KickScope, MODERATION_MODULE_ID,
    command::{Accept, ChangeDisplayName, Kick, ModerationCommand, SendToWaitingRoom},
    event::{DebriefingStarted, DisplayNameChanged, ModerationError, ModerationEvent},
    state::{ModerationState, ModeratorJoinInfo, WaitingParticipantPeerData},
};
use opentalk_types_common::{modules::ModuleId, users::DisplayName};
use opentalk_types_signaling::ParticipantId;
use tokio::sync::oneshot;

pub struct ModerationModule;

impl SignalingModule for ModerationModule {
    const NAMESPACE: ModuleId = MODERATION_MODULE_ID;

    type Incoming = ModerationCommand;

    type Outgoing = ModerationEvent;

    type Internal = NoOp;

    type Loopback =
        Result<Result<MicrophoneRestrictionState, MicrophoneRestrictionError>, ChannelDroppedError>;

    type JoinInfo = ModerationState;

    type PeerJoinInfo = ();

    type Error = ModerationError;

    fn init(_init_data: SignalingModuleInitData) -> Option<Self> {
        Some(Self)
    }

    #[allow(unused_variables)]
    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Self::Error>> {
        let moderator_data = if ctx.is_moderator(participant_id) {
            let info = ModeratorJoinInfo {
                waiting_room_enabled: ctx.room_task_info.room.waiting_room,
                waiting_room_participants: ctx
                    .waiting_participants
                    .iter()
                    .map(WaitingParticipantPeerData::from)
                    .collect(),
            };
            Some(info)
        } else {
            None
        };

        let join_info = ModuleJoinData {
            join_success: Some(ModerationState { moderator_data }),
            ..Default::default()
        };
        Ok(join_info)
    }

    #[allow(unused_variables)]
    fn on_participant_disconnected(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        Ok(())
    }

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        _connection_id: ConnectionId,
        content: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match content {
            ModerationCommand::Kick(Kick { target }) => self.kick_participant(ctx, sender, target),
            ModerationCommand::Debrief(kick_scope) => self.debrief(ctx, sender, kick_scope),
            ModerationCommand::EnableWaitingRoom => Ok(self.set_waiting_room_enabled(ctx, true)?),
            ModerationCommand::Accept(Accept { target }) => {
                Self::accept_waiting_room_participant(ctx, sender, target)
            }
            ModerationCommand::DisableWaitingRoom => Ok(self.set_waiting_room_enabled(ctx, false)?),
            ModerationCommand::SendToWaitingRoom(SendToWaitingRoom { target }) => {
                self.send_to_waiting_room(ctx, sender, target)
            }
            ModerationCommand::ChangeDisplayName(ChangeDisplayName { new_name, target }) => {
                self.change_display_name(ctx, sender, new_name, target)
            }
            ModerationCommand::EnableMicrophoneRestrictions {
                unrestricted_participants,
            } => self.update_microphone_restrictions(
                ctx,
                sender,
                MicrophoneRestrictionState::Enabled {
                    unrestricted_participants,
                },
            ),
            ModerationCommand::DisableMicrophoneRestrictions => self
                .update_microphone_restrictions(ctx, sender, MicrophoneRestrictionState::Disabled),
        }
    }

    fn on_loopback_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: Self::Loopback,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        let Ok(event) = event else {
            ctx.send_ws_message(
                ctx.participants.in_room(ctx.room).ids(),
                ModerationEvent::Error(ModerationError::Internal),
            )?;
            return Ok(());
        };

        match event {
            Ok(state) => self.notify_microphone_restrictions_updated(ctx, &state)?,
            Err(MicrophoneRestrictionError { sender, error }) => {
                ctx.send_ws_message([sender], ModerationEvent::Error(error.into()))?;
            }
        }
        Ok(())
    }
}

impl ModerationModule {
    fn kick_participant(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        target: ParticipantId,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        if !ctx.is_moderator(sender) {
            return Err(ModerationError::InsufficientPermissions.into());
        }

        if ctx.participants.connected().get(&target).is_none() {
            return Err(ModerationError::UnknownParticipant.into());
        }

        ctx.send_ws_message([target], ModerationEvent::Kicked)?;
        ctx.kick_participants(Vec::from_iter([target]));

        Ok(())
    }

    fn debrief(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        scope: KickScope,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        if !ctx.is_moderator(sender) {
            return Err(ModerationError::InsufficientPermissions.into());
        }

        let (kicked, not_kicked) =
            Self::split_by_kick_scope(&ctx.participants.all_unfiltered, scope);

        ctx.send_ws_message(
            not_kicked,
            ModerationEvent::DebriefingStarted(DebriefingStarted { issued_by: sender }),
        )?;

        if !ctx.room_task_info.room.waiting_room {
            self.set_waiting_room_enabled(ctx, true)?;
        }

        ctx.send_ws_message(kicked.clone(), ModerationEvent::Kicked)?;
        ctx.kick_participants(kicked);

        Ok(())
    }

    fn split_by_kick_scope(
        participants: &HashMap<ParticipantId, ParticipantState>,
        scope: KickScope,
    ) -> (Vec<ParticipantId>, Vec<ParticipantId>) {
        let mut kicked = Vec::new();
        let mut not_kicked = Vec::new();

        for (id, state) in participants {
            if scope.kicks(state.role, &state.kind) {
                kicked.push(*id);
            } else {
                not_kicked.push(*id);
            }
        }

        (kicked, not_kicked)
    }

    fn accept_waiting_room_participant(
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        target: ParticipantId,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        if !ctx.is_moderator(sender) {
            tracing::debug!("Insufficient permissions");
            return Err(ModerationError::InsufficientPermissions.into());
        }

        let Some(participant) = ctx.waiting_participants.get_mut(&target) else {
            tracing::debug!(
                "Failed to send `accept` to waiting participant: participant not known ({target})"
            );
            return Err(ModerationError::NotWaiting.into());
        };

        participant.accepted = true;

        tracing::trace!("accept participant: {target}");
        ctx.send_ws_message_to_waiting_room([target], ModerationEvent::Accepted)?;

        Ok(())
    }

    fn set_waiting_room_enabled(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        enabled: bool,
    ) -> Result<(), FatalError> {
        ctx.room_task_info.room.waiting_room = enabled;
        let event = if enabled {
            ModerationEvent::WaitingRoomEnabled
        } else {
            ModerationEvent::WaitingRoomDisabled
        };
        ctx.send_ws_message(ctx.participants.connected().ids(), event)
    }

    fn send_to_waiting_room(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        target: ParticipantId,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        if !ctx.is_moderator(sender) {
            return Err(ModerationError::InsufficientPermissions.into());
        }

        if ctx.is_room_owner(target) {
            return Err(ModerationError::CannotSendRoomOwnerToWaitingRoom.into());
        }

        if !ctx.room_task_info.room.waiting_room {
            self.set_waiting_room_enabled(ctx, true)?;
        }

        ctx.send_ws_message([target], ModerationEvent::SentToWaitingRoom)?;
        ctx.move_to_waiting_room(target);

        Ok(())
    }

    fn change_display_name(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        mut new_name: DisplayName,
        target: ParticipantId,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        if !ctx.is_moderator(sender) {
            return Err(ModerationError::InsufficientPermissions.into());
        }

        if new_name.as_str().trim().is_empty() || new_name.len() > 100 {
            return Err(ModerationError::InvalidDisplayName.into());
        }
        // Sanitize the display name
        new_name = DisplayName::from_str_lossy(new_name.as_str());

        let Some(participant) = ctx.participants.all_unfiltered.get_mut(&target) else {
            return Err(ModerationError::UnknownParticipant.into());
        };

        let ClientKind::Guest { display_name } = &mut participant.kind else {
            return Err(ModerationError::CannotChangeNameOfRegisteredUsers.into());
        };

        let old_name = mem::replace(display_name, new_name.clone());

        ctx.send_ws_message(
            ctx.participants.connected().ids(),
            ModerationEvent::DisplayNameChanged(DisplayNameChanged {
                target,
                issued_by: sender,
                old_name,
                new_name,
            }),
        )?;

        Ok(())
    }

    fn update_microphone_restrictions(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        new_state: MicrophoneRestrictionState,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        if !ctx.is_moderator(sender) {
            return Err(ModerationError::InsufficientPermissions.into());
        }

        let (tx, rx) = oneshot::channel();

        ctx.send_internal_command::<LiveKitModule>(LiveKitInternal::UpdateMicrophoneRestrictions {
            sender,
            new_state,
            return_channel: tx,
        });

        ctx.recv_loopback(rx, |result| result);

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, ctx), fields(room= ?ctx.room))]
    pub fn notify_microphone_restrictions_updated(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        microphone_restrictions: &MicrophoneRestrictionState,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        match microphone_restrictions {
            MicrophoneRestrictionState::Disabled => {
                ctx.send_ws_message(
                    ctx.participants.connected().room(ctx.room).ids(),
                    ModerationEvent::MicrophoneRestrictionsDisabled,
                )?;
            }
            MicrophoneRestrictionState::Enabled {
                unrestricted_participants,
            } => {
                ctx.send_ws_message(
                    ctx.participants.connected().room(ctx.room).ids(),
                    ModerationEvent::MicrophoneRestrictionsEnabled {
                        unrestricted_participants: unrestricted_participants
                            .iter()
                            .copied()
                            .collect(),
                    },
                )?;
            }
        }

        Ok(())
    }
}
