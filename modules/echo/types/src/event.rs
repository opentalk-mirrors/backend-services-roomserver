// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use serde::{Deserialize, Serialize};

use crate::error::EchoError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum EchoEvent {
    Pong,
    DelayedPong,
    Error(EchoError),
    Replication(Replication),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "replicated_command", rename_all = "snake_case")]
pub enum Replication {
    ReplicatedPing,
}

impl From<EchoError> for EchoEvent {
    fn from(err: EchoError) -> Self {
        Self::Error(err)
    }
}
