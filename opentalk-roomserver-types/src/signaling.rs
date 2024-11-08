// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::BTreeSet;

use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalingEvent {
    pub namespace: String,
    pub content: serde_json::Value,
}

pub enum MessageTarget {
    AllParticipantsInRoom,
    // Group(GroupName),
    Participant(ParticipantId),
    Participants(BTreeSet<ParticipantId>),
}

/// Errors that are reported back to the client via the signaling socket.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "error")]
pub enum SignalingError {
    /// The received message was not valid JSON.
    InvalidJson { message: String },
}

impl From<serde_json::Error> for SignalingError {
    fn from(error: serde_json::Error) -> Self {
        Self::InvalidJson {
            message: format!("Failed to deserialize message: {}", error),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignalingCommand {
    pub namespace: ModuleId,
    pub transaction_id: Option<u64>,
    pub content: serde_json::Value,

    /// Unknown fields. This should always be empty. If not a warning should be triggered.
    #[serde(flatten, skip_serializing)]
    unknown_fields: serde_json::Value,
}

impl SignalingCommand {
    pub fn has_unknown_fields(&self) -> bool {
        !self.unknown_fields.is_null()
    }

    /// Print an info log with additional unexpected fields.
    ///
    /// NOTE: `unknown_fields` should be a [`serde_json::Value::Object`] variant. This
    /// should be the case when collecting additional fields in a struct definition
    /// (see [`SignalingCommand`] for an example)
    pub fn unknown_fields(&self) -> Option<Vec<String>> {
        if !self.has_unknown_fields() {
            return None;
        }

        match &self.unknown_fields {
            serde_json::Value::Object(map) => {
                let keys: Vec<_> = map.keys().cloned().collect();
                Some(keys)
            }
            other => {
                // This branch should be unreachable since we collect additional
                // fields in a struct, which results in a json object
                Some(vec![format!("<Unexpected type: {}>", other)])
            }
        }
    }
}
