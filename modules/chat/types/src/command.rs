// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use opentalk_types_signaling_chat::command::{SendMessage, SetLastSeenTimestamp};

use crate::event::ChatEvent;

/// Commands for the `chat` namespace
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ChatCommand {
    /// Enable chat messaging
    EnableChat,

    /// Disable chat messaging
    DisableChat,

    /// Send chat message
    SendMessage(SendMessage),

    /// Clear chat history
    ClearHistory,

    /// Set last seen timestamp
    SetLastSeenTimestamp(SetLastSeenTimestamp),
}

impl CreateReplica<ChatEvent> for ChatCommand {
    fn replicate(&self) -> Option<ChatEvent> {
        None
    }
}

#[cfg(test)]
mod tests {
    use opentalk_types_common::time::Timestamp;
    use opentalk_types_signaling_chat::Scope;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn serialize_enable_chat() {
        let command = ChatCommand::EnableChat;
        let serialized = serde_json::to_string(&command).expect("Serialization failed");
        assert_eq!(serialized, r#"{"action":"enable_chat"}"#);
    }

    #[test]
    fn deserialize_enable_chat() {
        let json_data = r#"{"action":"enable_chat"}"#;
        let deserialized: ChatCommand =
            serde_json::from_str(json_data).expect("Deserialization failed");
        assert!(matches!(deserialized, ChatCommand::EnableChat));
    }

    #[test]
    fn serialize_send_message() {
        let send_message = SendMessage {
            content: "test message".to_string(),
            scope: Scope::Global,
        };
        let command = ChatCommand::SendMessage(send_message.clone());
        let serialized = serde_json::to_string(&command).expect("Serialization failed");
        let expected = format!(
            r#"{{"action":"send_message","content":"{}","scope":"global"}}"#,
            send_message.content
        );
        assert_eq!(serialized, expected);
    }

    #[test]
    fn deserialize_send_message() {
        let json_data = r#"{"action":"send_message","scope":"global","content":"Hello!"}"#;
        let deserialized: ChatCommand =
            serde_json::from_str(json_data).expect("Deserialization failed");
        if let ChatCommand::SendMessage(send_message) = deserialized {
            assert_eq!(send_message.content, "Hello!");
            assert_eq!(send_message.scope, Scope::Global);
        } else {
            panic!("Expected ChatCommand::SendMessage");
        }
    }

    #[test]
    fn serialize_set_last_seen_timestamp() {
        let set_last_seen = SetLastSeenTimestamp {
            scope: Scope::Global,
            timestamp: Timestamp::unix_epoch(),
        };
        let command = ChatCommand::SetLastSeenTimestamp(set_last_seen.clone());
        let serialized = serde_json::to_string(&command).expect("Serialization failed");
        let expected = r#"{"action":"set_last_seen_timestamp","scope":"global","timestamp":"1970-01-01T00:00:00Z"}"#;
        assert_eq!(serialized, expected);
    }

    #[test]
    fn deserialize_set_last_seen_timestamp() {
        let json_data = r#"{"action":"set_last_seen_timestamp","scope":"global","timestamp":"2025-05-13T12:18:36.915786041Z"}"#;
        let deserialized: ChatCommand =
            serde_json::from_str(json_data).expect("Deserialization failed");
        if let ChatCommand::SetLastSeenTimestamp(set_last_seen) = deserialized {
            assert_eq!(set_last_seen.scope, Scope::Global);
        } else {
            panic!("Expected ChatCommand::SetLastSeenTimestamp");
        }
    }
}
