// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::time::Duration;

use opentalk_roomserver_signaling::{
    module_context::ModuleContext,
    signaling_module::{JoinInfo, SignalingModule, SignalingModuleInitData},
};
use opentalk_roomserver_types::{
    connection_id::ConnectionId,
    signaling::module_error::{FatalError, SignalingModuleError},
};
use opentalk_roomserver_types_ping::{
    PING_MODULE_ID, command::PingCommand, error::PingError, event::PingEvent,
};
use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::ParticipantId;

pub struct PingModule;

impl SignalingModule for PingModule {
    const NAMESPACE: ModuleId = PING_MODULE_ID;

    type Incoming = PingCommand;

    type Outgoing = PingEvent;

    type Loopback = DelayedPingCompleted;

    type JoinInfo = ();

    type PeerJoinInfo = String;

    type Error = PingError;

    fn init(_init_data: SignalingModuleInitData) -> Option<Self> {
        Some(Self)
    }

    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<JoinInfo<Self>, SignalingModuleError<Self::Error>> {
        log::info!("Participant {participant_id} connected");
        let mut join_info = JoinInfo::default();

        for (participant_id, ..) in ctx.participants.connected().iter() {
            join_info
                .peer
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
        log::info!("Participant {participant_id} disconnected");
        Ok(())
    }

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
        content: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match content {
            PingCommand::Ping | PingCommand::ReplicatedPing => {
                ctx.send_ws_message([participant_id], PingEvent::Pong)?
            }
            PingCommand::AsyncDelayedPing { delay } => {
                ctx.spawn(Self::handle_async_ping_delayed(participant_id, delay));
            }
            PingCommand::Die => {
                return Err(
                    FatalError(anyhow::anyhow!("Dying as requested, cya later alligator")).into(),
                );
            }
        }
        Ok(())
    }

    fn on_loopback_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: Self::Loopback,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        ctx.send_ws_message([event.0], PingEvent::DelayedPong)
            .unwrap();
        Ok(())
    }
}

impl PingModule {
    async fn handle_async_ping_delayed(
        participant_id: ParticipantId,
        delay: Duration,
    ) -> DelayedPingCompleted {
        tokio::time::sleep(delay).await;
        DelayedPingCompleted(participant_id)
    }
}

pub struct DelayedPingCompleted(ParticipantId);
