// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_common::modules::{module_id, ModuleId};
use serde::{Deserialize, Serialize};

pub const NAMESPACE: ModuleId = module_id!("error");

/// Errors that are reported back to the client via the signaling socket.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "error")]
pub enum SignalingError {
    /// The requested namespace is unknown to the room server
    UnknownNamespace {
        invalid_namespace: String,
    },

    /// The received message was not valid JSON.
    InvalidJson {
        message: String,
    },

    // A non-specific internal error
    Internal,
}

impl From<serde_json::Error> for SignalingError {
    fn from(error: serde_json::Error) -> Self {
        Self::InvalidJson {
            message: format!("Failed to deserialize message: {error}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::SignalingError;

    #[test]
    fn unknown_namespace() {
        let val = SignalingError::UnknownNamespace {
            invalid_namespace: "test".into(),
        };

        let val = serde_json::to_value(val).unwrap();

        assert_eq!(
            val,
            json!( {
                "error": "unknown_namespace",
                "invalid_namespace": "test"
            }),
        );
    }
}
