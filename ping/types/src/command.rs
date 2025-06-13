// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::time::Duration;

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use serde::{Deserialize, Serialize};

use crate::event::{Event, Replication};

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
    /// A ping that will result in a [`PingError`](crate::error::PingError)
    PingError,
    /// Ping all participants
    Broadcast,
    /// Request the ping module to die by returning a
    /// [`FatalError`](opentalk_roomserver_types::signaling::module_error::FatalError)
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
