// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::time::Duration;

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use serde::{Deserialize, Serialize};

use crate::event::{PingEvent, Replication};

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum PingCommand {
    /// A normal ping
    Ping,
    /// A ping with delayed response
    AsyncDelayedPing {
        /// The duration that the pong is delayed for.
        #[serde(with = "opentalk_types_common::utils::duration_seconds")]
        delay: Duration,
    },
    /// Request the ping module to die by returning a
    /// [`FatalError`](opentalk_roomserver_types::signaling::module_error::FatalError)
    Die,
    /// A ping where the command gets replicated
    ReplicatedPing,
}

impl CreateReplica<PingEvent> for PingCommand {
    fn replicate(&self) -> Option<PingEvent> {
        match self {
            PingCommand::ReplicatedPing => {
                Some(PingEvent::Replication(Replication::ReplicatedPing))
            }
            _ => None,
        }
    }
}
