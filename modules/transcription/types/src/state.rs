// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptionStatus {
    Inactive,
    Requested,
    Running,
}

/// The state of the `transcription` module.
///
/// This struct is sent to the participant in the `join_success` message
/// when they successfully join the meeting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranscriptionState {
    pub status: TranscriptionStatus,
}

impl opentalk_types_signaling::SignalingModuleFrontendData for TranscriptionState {
    const NAMESPACE: Option<opentalk_types_common::modules::ModuleId> =
        Some(crate::TRANSCRIPTION_MODULE_ID);
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use super::*;

    #[test]
    fn serialize_inactive_state() {
        let state = TranscriptionState {
            status: TranscriptionStatus::Inactive,
        };

        assert_json_snapshot!(state, @ r#"
        {
          "status": "inactive"
        }
        "#);
    }

    #[test]
    fn serialize_requested_state() {
        let state = TranscriptionState {
            status: TranscriptionStatus::Requested,
        };

        assert_json_snapshot!(state, @ r#"
        {
          "status": "requested"
        }
        "#);
    }

    #[test]
    fn serialize_running_state() {
        let state = TranscriptionState {
            status: TranscriptionStatus::Running,
        };

        assert_json_snapshot!(state, @ r#"
        {
          "status": "running"
        }
        "#);
    }
}
