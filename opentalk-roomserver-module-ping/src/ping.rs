// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! TODO: PoC demo module, to be removed
use std::{thread, time::Duration};

use opentalk_roomserver_signaling::{
    module_context::ModuleContext,
    signaling_module::{
        FatalError, JoinInfo, ModuleError, SignalingModule, SignalingModuleError,
        SignalingModuleInitData,
    },
};
use opentalk_roomserver_types::connection_id::ConnectionId;
use opentalk_types_common::modules::{module_id, ModuleId};
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

const MODULE_ID: ModuleId = module_id!("ping");

pub struct PingModule;

impl SignalingModule for PingModule {
    const NAMESPACE: ModuleId = MODULE_ID;

    type Incoming = Command;

    type Outgoing = Event;

    type Loopback = DelayedPingCompleted;

    type JoinInfo = ();

    type PeerJoinInfo = String;

    type Error = PingError;

    async fn init(_init_data: SignalingModuleInitData) -> Option<Self> {
        Some(Self)
    }

    async fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<JoinInfo<Self>, SignalingModuleError<Self::Error>> {
        log::info!("Participant {participant_id} connected");
        let mut join_info = JoinInfo::default();

        for (participant_id, ..) in ctx.participants.connected() {
            join_info
                .peer
                .insert(*participant_id, format!("Hello {participant_id}"))?;
        }

        Ok(join_info)
    }

    async fn on_participant_disconnected(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        log::info!("Participant {participant_id} disconnected");
        Ok(())
    }

    async fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
        content: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match content {
            Command::Ping => ctx.send_ws_message(participant_id, Event::Pong)?,
            Command::BlockingDelayedPing => {
                ctx.spawn_blocking(move || Self::handle_ping_delayed(participant_id));
            }
            Command::AsyncDelayedPing => {
                ctx.spawn(Self::handle_async_ping_delayed(participant_id));
            }
            Command::PingError => Self::ping_error()?,
            Command::Die => {
                return Err(
                    FatalError(anyhow::anyhow!("Dying as requested, cya later alligator")).into(),
                );
            }
        }
        Ok(())
    }

    async fn on_loopback_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: Self::Loopback,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        ctx.send_ws_message(event.0, Event::DelayedPong).unwrap();
        Ok(())
    }
}

impl PingModule {
    fn handle_ping_delayed(participant_id: ParticipantId) -> DelayedPingCompleted {
        thread::sleep(Duration::from_secs(3));
        DelayedPingCompleted(participant_id)
    }

    async fn handle_async_ping_delayed(participant_id: ParticipantId) -> DelayedPingCompleted {
        tokio::time::sleep(Duration::from_secs(3)).await;
        DelayedPingCompleted(participant_id)
    }

    fn ping_error() -> Result<(), PingError> {
        Err(PingError)
    }
}

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Command {
    /// A normal ping
    Ping,
    /// A ping with delayed response
    BlockingDelayedPing,
    /// A ping with delayed response
    AsyncDelayedPing,
    /// A ping that will result in a [`PingError`]
    PingError,
    /// Request the ping module to die by returning a [`FatalError`]
    Die,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum Event {
    Pong,
    DelayedPong,
    Error(PingError),
}

impl From<PingError> for Event {
    fn from(err: PingError) -> Self {
        Self::Error(err)
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct PingError;

impl ModuleError for PingError {}

pub struct DelayedPingCompleted(ParticipantId);
