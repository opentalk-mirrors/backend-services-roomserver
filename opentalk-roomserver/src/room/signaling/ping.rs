// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! TODO: PoC demo module, to be removed
use opentalk_types_common::modules::{module_id, ModuleId};
use serde::{Deserialize, Serialize};

use super::{
    signaling_module::SignalingModuleInitData, ModuleContext, SignalingEvent, SignalingModule,
};

const MODULE_ID: ModuleId = module_id!("ping");

pub struct PingModule;

#[async_trait::async_trait]
impl SignalingModule for PingModule {
    const NAMESPACE: ModuleId = MODULE_ID;

    type Incoming = Command;

    type Outgoing = Event;

    async fn init(_init_data: SignalingModuleInitData) -> Option<Self> {
        Some(Self)
    }

    async fn on_event(&mut self, ctx: &mut ModuleContext<'_, Self>, event: SignalingEvent<Self>) {
        match event {
            SignalingEvent::WebsocketMessage { sender, content } => match content {
                Command::Ping => ctx.send_ws_message(sender, Event::Pong).await.unwrap(),
            },
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Command {
    /// A normal ping
    Ping,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum Event {
    Pong,
}
