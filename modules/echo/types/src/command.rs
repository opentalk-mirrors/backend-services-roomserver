// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use serde::{Deserialize, Serialize};

use crate::event::{EchoEvent, Replication};

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum EchoCommand {
    /// A normal ping
    Ping,
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
