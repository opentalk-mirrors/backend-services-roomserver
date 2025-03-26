// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::BTreeSet;

use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalingEvent {
    pub namespace: ModuleId,
    pub content: serde_json::Value,
}

pub enum MessageTarget {
    AllParticipantsInRoom,
    // Group(GroupName),
    Participant(ParticipantId),
    Participants(BTreeSet<ParticipantId>),
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
    pub fn new(
        namespace: ModuleId,
        transaction_id: Option<u64>,
        content: serde_json::Value,
    ) -> Self {
        Self {
            namespace,
            transaction_id,
            content,
            unknown_fields: serde_json::Value::Object(Default::default()),
        }
    }

    #[must_use]
    pub fn has_unknown_fields(&self) -> bool {
        !self.unknown_fields.is_null()
    }

    /// Print an info log with additional unexpected fields.
    ///
    /// NOTE: `unknown_fields` should be a [`serde_json::Value::Object`] variant. This
    /// should be the case when collecting additional fields in a struct definition
    /// (see [`SignalingCommand`] for an example)
    #[must_use]
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
