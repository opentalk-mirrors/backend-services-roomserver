// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{thread, time::Duration};

use opentalk_roomserver_signaling::{
    module_context::ModuleContext,
    signaling_module::{CreateReplica, JoinInfo, SignalingModule, SignalingModuleInitData},
};
use opentalk_roomserver_types::{
    connection_id::ConnectionId,
    signaling::module_error::{FatalError, ModuleError, SignalingModuleError},
};
use opentalk_types_common::modules::{ModuleId, module_id};
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
            Command::Ping | Command::ReplicatedPing => {
                ctx.send_ws_message([participant_id], Event::Pong)?
            }
            Command::BlockingDelayedPing { delay } => {
                ctx.spawn_blocking(move || Self::handle_ping_delayed(participant_id, delay));
            }
            Command::AsyncDelayedPing { delay } => {
                ctx.spawn(Self::handle_async_ping_delayed(participant_id, delay));
            }
            Command::PingError => Self::ping_error()?,
            Command::Broadcast => ctx.send_ws_message(
                ctx.participants.connected().iter().map(|(id, _)| *id),
                Event::Pong,
            )?,
            Command::Die => {
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
        ctx.send_ws_message([event.0], Event::DelayedPong).unwrap();
        Ok(())
    }
}

impl PingModule {
    fn handle_ping_delayed(participant_id: ParticipantId, delay: Duration) -> DelayedPingCompleted {
        thread::sleep(delay);
        DelayedPingCompleted(participant_id)
    }

    async fn handle_async_ping_delayed(
        participant_id: ParticipantId,
        delay: Duration,
    ) -> DelayedPingCompleted {
        tokio::time::sleep(delay).await;
        DelayedPingCompleted(participant_id)
    }

    fn ping_error() -> Result<(), PingError> {
        Err(PingError)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Command {
    /// A normal ping
    Ping,
    /// A ping with delayed response
    BlockingDelayedPing {
        /// The duration that the pong is delayed for.
        #[serde(with = "opentalk_types_common::utils::duration_seconds")]
        delay: Duration,
    },
    /// A ping with delayed response
    AsyncDelayedPing {
        /// The duration that the pong is delayed for.
        #[serde(with = "opentalk_types_common::utils::duration_seconds")]
        delay: Duration,
    },
    /// A ping that will result in a [`PingError`]
    PingError,
    /// Ping all participants
    Broadcast,
    /// Request the ping module to die by returning a [`FatalError`]
    Die,
    /// A ping where the command gets replicated
    ReplicatedPing,
}

impl CreateReplica<Event> for Command {
    fn replicate(&self) -> Option<Event> {
        match self {
            Command::ReplicatedPing => Some(Event::Replication(Replication::ReplicatedPing)),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum Event {
    Pong,
    DelayedPong,
    Error(PingError),
    Replication(Replication),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "replicated_command", rename_all = "snake_case")]
pub enum Replication {
    ReplicatedPing,
}

impl From<PingError> for Event {
    fn from(err: PingError) -> Self {
        Self::Error(err)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PingError;

impl ModuleError for PingError {}

pub struct DelayedPingCompleted(ParticipantId);
