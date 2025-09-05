// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_signaling::{
    module_context::ModuleContext,
    signaling_module::{ModuleJoinData, NoOp, SignalingModule, SignalingModuleInitData},
};
use opentalk_roomserver_types::{
    connection_id::ConnectionId, signaling::module_error::SignalingModuleError,
};
use opentalk_roomserver_types_echo::{
    ECHO_MODULE_ID, command::EchoCommand, error::EchoError, event::EchoEvent,
};
use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::ParticipantId;

pub struct EchoModule;

impl SignalingModule for EchoModule {
    const NAMESPACE: ModuleId = ECHO_MODULE_ID;

    type Incoming = EchoCommand;

    type Outgoing = EchoEvent;

    type Internal = NoOp;

    type Loopback = ();

    type JoinInfo = ();

    type PeerJoinInfo = String;

    type Error = EchoError;

    fn init(_init_data: SignalingModuleInitData) -> Option<Self> {
        Some(Self)
    }

    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Self::Error>> {
        tracing::info!("Participant {participant_id} connected");
        let mut join_info = ModuleJoinData::default();

        for (participant_id, ..) in ctx.participants.connected().iter() {
            join_info
                .peer_events
                .insert(*participant_id, format!("Hello {participant_id}"))?;
        }

        Ok(join_info)
    }

    fn on_participant_disconnected(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        tracing::info!("Participant {participant_id} disconnected");
        Ok(())
    }

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
        payload: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match payload {
            EchoCommand::Ping => ctx.send_ws_message([participant_id], EchoEvent::Pong)?,
        }
        Ok(())
    }

    fn on_websocket_message_waiting_room(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        _connection_id: ConnectionId,
        payload: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match payload {
            EchoCommand::Ping => ctx.send_ws_message_to_waiting_room([sender], EchoEvent::Pong)?,
        }
        Ok(())
    }
}
