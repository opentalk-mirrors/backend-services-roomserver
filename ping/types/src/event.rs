// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use serde::{Deserialize, Serialize};

use crate::error::PingError;

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
