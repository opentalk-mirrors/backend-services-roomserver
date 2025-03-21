// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! TODO: PoC demo module, to be removed
use std::{thread, time::Duration};

use opentalk_types_common::modules::{module_id, ModuleId};
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use super::{
    signaling_module::{FatalError, ModuleError, SignalingModuleError, SignalingModuleInitData},
    ModuleContext, SignalingEvent, SignalingModule,
};

const MODULE_ID: ModuleId = module_id!("ping");

pub struct PingModule;

#[async_trait::async_trait]
impl SignalingModule for PingModule {
    const NAMESPACE: ModuleId = MODULE_ID;

    type Incoming = Command;

    type Outgoing = Event;

    type Loopback = LoopbackEvent;

    type Error = PingError;

    async fn init(_init_data: SignalingModuleInitData) -> Option<Self> {
        Some(Self)
    }

    async fn on_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: SignalingEvent<Self>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match event {
            SignalingEvent::WebsocketMessage { sender, content } => match content {
                Command::Ping => ctx.send_ws_message(sender, Event::Pong).await?,
                Command::BlockingDelayedPing => {
                    ctx.spawn_blocking(move || Self::handle_ping_delayed(sender));
                }
                Command::AsyncDelayedPing => {
                    ctx.spawn(Self::handle_async_ping_delayed(sender));
                }
                Command::PingError => Self::ping_error()?,
                Command::Die => {
                    return Err(FatalError(anyhow::anyhow!(
                        "Dying as requested, cya later alligator"
                    ))
                    .into());
                }
            },
            SignalingEvent::LoopbackMessage(msg) => match msg {
                LoopbackEvent::DelayedPingCompleted(participant_id) => {
                    ctx.send_ws_message(participant_id, Event::DelayedPong)
                        .await
                        .unwrap();
                }
            },
        }

        Ok(())
    }
}

impl PingModule {
    fn handle_ping_delayed(participant_id: ParticipantId) -> LoopbackEvent {
        thread::sleep(Duration::from_secs(3));
        LoopbackEvent::DelayedPingCompleted(participant_id)
    }

    async fn handle_async_ping_delayed(participant_id: ParticipantId) -> LoopbackEvent {
        tokio::time::sleep(Duration::from_secs(3)).await;
        LoopbackEvent::DelayedPingCompleted(participant_id)
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

pub enum LoopbackEvent {
    DelayedPingCompleted(ParticipantId),
}
