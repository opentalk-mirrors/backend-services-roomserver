// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling state for the `chat` namespace

use std::collections::BTreeMap;

use opentalk_types_common::{time::Timestamp, users::GroupName};
use opentalk_types_signaling::ParticipantId;

use crate::state::{GroupHistory, PrivateHistory, StoredMessage};

/// The state of the `chat` module
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChatState {
    /// Is the chat module enabled
    pub enabled: bool,

    /// Chat history for the room
    pub room_history: Vec<StoredMessage>,

    /// All group chat history in the room
    pub groups_history: Vec<GroupHistory>,

    /// All private chat history in the room
    pub private_history: Vec<PrivateHistory>,

    /// Timestamp for last time someone read a message
    pub last_seen_timestamp_global: Option<Timestamp>,

    /// Timestamp for last time someone read a private message
    pub last_seen_timestamps_private: BTreeMap<ParticipantId, Timestamp>,

    /// Timestamp for last time someone read a group message
    pub last_seen_timestamps_group: BTreeMap<GroupName, Timestamp>,
}

#[cfg(feature = "serde")]
impl opentalk_types_signaling::SignalingModuleFrontendData for ChatState {
    const NAMESPACE: Option<opentalk_types_common::modules::ModuleId> = Some(crate::MODULE_ID);
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use std::str::FromStr;

    use chrono::DateTime;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::{MessageId, Scope};

    #[test]
    fn server_message() {
        let expected = json!({
            "id":"00000000-0000-0000-0000-000000000000",
            "source":"00000000-0000-0000-0000-000000000000",
            "timestamp":"2021-06-24T14:00:11.873753715Z",
            "content":"Hello All!",
            "scope":"global",
        });

        let produced = serde_json::to_value(StoredMessage {
            id: MessageId::nil(),
            source: ParticipantId::nil(),
            timestamp: DateTime::from_str("2021-06-24T14:00:11.873753715Z")
                .unwrap()
                .into(),
            content: "Hello All!".to_string(),
            scope: Scope::Global,
        })
        .unwrap();

        assert_eq!(expected, produced);
    }
}
