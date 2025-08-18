// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::time::Duration;

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use serde::{Deserialize, Serialize};

use crate::event::{EchoEvent, Replication};

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum EchoCommand {
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

impl CreateReplica<EchoEvent> for EchoCommand {
    fn replicate(&self) -> Option<EchoEvent> {
        match self {
            EchoCommand::ReplicatedPing => {
                Some(EchoEvent::Replication(Replication::ReplicatedPing))
            }
            _ => None,
        }
    }
}
