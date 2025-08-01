// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::HashMap;

use opentalk_roomserver_signaling::{
    module_context::ModuleContext,
    participant_state::ParticipantState,
    signaling_module::{JoinInfo, NoOp, SignalingModule, SignalingModuleInitData},
};
use opentalk_roomserver_types::{
    connection_id::ConnectionId,
    signaling::module_error::{FatalError, SignalingModuleError},
};
use opentalk_roomserver_types_moderation::{
    KickScope, MODERATION_MODULE_ID,
    command::{Accept, Kick, ModerationCommand},
    event::{DebriefingStarted, ModerationError, ModerationEvent},
};
use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::ParticipantId;

pub struct ModerationModule;

impl SignalingModule for ModerationModule {
    const NAMESPACE: ModuleId = MODERATION_MODULE_ID;

    type Incoming = ModerationCommand;

    type Outgoing = ModerationEvent;

    type Internal = NoOp;

    type Loopback = ();

    type JoinInfo = ();

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
    ) -> Result<JoinInfo<Self>, SignalingModuleError<Self::Error>> {
        Ok(JoinInfo::default())
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
            ModerationCommand::Accept(Accept { target }) => {
                Self::accept_waiting_room_participant(ctx, sender, target)
            }
        }
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

        self.set_waiting_room_enabled(ctx, true)?;

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
            if scope.kicks(state.role, state.kind) {
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
        let connections: Vec<ConnectionId> = participant.connections.keys().copied().collect();
        // Participants in the waiting room do not have a participant state,
        // from which the connection ids could be determined, so we have to use
        // send_ws_message_to_connections().
        ctx.send_ws_message_to_connections(connections, ModerationEvent::Accepted)?;

        Ok(())
    }

    fn set_waiting_room_enabled(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        enabled: bool,
    ) -> Result<(), FatalError> {
        if ctx.room_task_info.room.waiting_room == enabled {
            return Ok(());
        }

        ctx.room_task_info.room.waiting_room = enabled;
        let event = if enabled {
            ModerationEvent::WaitingRoomEnabled
        } else {
            ModerationEvent::WaitingRoomDisabled
        };
        ctx.send_ws_message(ctx.participants.connected().ids(), event)
    }
}
