// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use serde::{Deserialize, Serialize};

use crate::error::PingError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum PingEvent {
    Pong,
    DelayedPong,
    Error(PingError),
    Replication(Replication),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "replicated_command", rename_all = "snake_case")]
pub enum Replication {
    ReplicatedPing,
}

impl From<PingError> for PingEvent {
    fn from(err: PingError) -> Self {
        Self::Error(err)
    }
}
