// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_signaling::signaling_module::ModuleError;
use opentalk_types_signaling_chat::{
    command::SetLastSeenTimestamp,
    event::{ChatDisabled, ChatEnabled, HistoryCleared, MessageSent},
};

/// A chat event which occurred
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum ChatEvent {
    /// Chat event where chat was enabled see [ChatEnabled]
    ChatEnabled(ChatEnabled),

    /// Chat event where chat was disabled see [ChatDisabled]
    ChatDisabled(ChatDisabled),

    /// Chat event where a message was sent see [MessageSent]
    MessageSent(MessageSent),

    /// Chat event where history was cleared see [HistoryCleared]
    HistoryCleared(HistoryCleared),

    /// Chat event when last seen timestamp was set.
    SetLastSeenTimestamp(SetLastSeenTimestamp),

    /// Chat event which errored see [ChatError]
    Error(ChatError),
}

impl From<ChatEnabled> for ChatEvent {
    fn from(value: ChatEnabled) -> Self {
        Self::ChatEnabled(value)
    }
}

impl From<ChatDisabled> for ChatEvent {
    fn from(value: ChatDisabled) -> Self {
        Self::ChatDisabled(value)
    }
}

impl From<MessageSent> for ChatEvent {
    fn from(value: MessageSent) -> Self {
        Self::MessageSent(value)
    }
}

impl From<HistoryCleared> for ChatEvent {
    fn from(value: HistoryCleared) -> Self {
        Self::HistoryCleared(value)
    }
}

impl From<ChatError> for ChatEvent {
    fn from(value: ChatError) -> Self {
        Self::Error(value)
    }
}

impl From<SetLastSeenTimestamp> for ChatEvent {
    fn from(value: SetLastSeenTimestamp) -> Self {
        Self::SetLastSeenTimestamp(value)
    }
}

/// Errors from the `chat` module namespace
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum ChatError {
    /// Request while chat is disabled
    ChatDisabled,

    /// Requesting user has insufficient permissions
    InsufficientPermissions,
}

impl ModuleError for ChatError {}

#[cfg(test)]
mod tests {
    use opentalk_types_signaling::ParticipantId;
    use opentalk_types_signaling_chat::{
        MessageId, Scope,
        event::{ChatDisabled, ChatEnabled, HistoryCleared, MessageSent},
    };
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn serialize_chat_enabled() {
        let event = ChatEvent::ChatEnabled(ChatEnabled {
            issued_by: ParticipantId::from_u128(1),
        });
        let serialized = serde_json::to_string(&event).expect("Serialization failed");
        assert_eq!(
            serialized,
            r#"{"message":"chat_enabled","issued_by":"00000000-0000-0000-0000-000000000001"}"#
        );
    }

    #[test]
    fn deserialize_chat_enabled() {
        let json_data =
            r#"{"message":"chat_enabled","issued_by":"00000000-0000-0000-0000-000000000001"}"#;
        let deserialized: ChatEvent =
            serde_json::from_str(json_data).expect("Deserialization failed");
        if let ChatEvent::ChatEnabled(chat_enabled) = deserialized {
            assert_eq!(chat_enabled.issued_by, ParticipantId::from_u128(1));
        } else {
            panic!("Expected ChatEvent::ChatEnabled");
        }
    }

    #[test]
    fn serialize_chat_disabled() {
        let event = ChatEvent::ChatDisabled(ChatDisabled {
            issued_by: ParticipantId::from_u128(2),
        });
        let serialized = serde_json::to_string(&event).expect("Serialization failed");
        assert_eq!(
            serialized,
            r#"{"message":"chat_disabled","issued_by":"00000000-0000-0000-0000-000000000002"}"#
        );
    }

    #[test]
    fn deserialize_chat_disabled() {
        let json_data =
            r#"{"message":"chat_disabled","issued_by":"00000000-0000-0000-0000-000000000002"}"#;
        let deserialized: ChatEvent =
            serde_json::from_str(json_data).expect("Deserialization failed");
        if let ChatEvent::ChatDisabled(chat_disabled) = deserialized {
            assert_eq!(chat_disabled.issued_by, ParticipantId::from_u128(2));
        } else {
            panic!("Expected ChatEvent::ChatDisabled");
        }
    }

    #[test]
    fn serialize_message_sent() {
        let event = ChatEvent::MessageSent(MessageSent {
            id: MessageId::from_u128(123),
            source: ParticipantId::from_u128(3),
            content: "Hello, world!".to_string(),
            scope: Scope::Global,
        });
        let serialized = serde_json::to_string(&event).expect("Serialization failed");
        assert_eq!(
            serialized,
            r#"{"message":"message_sent","id":"00000000-0000-0000-0000-00000000007b","source":"00000000-0000-0000-0000-000000000003","content":"Hello, world!","scope":"global"}"#
        );
    }

    #[test]
    fn deserialize_message_sent() {
        let json_data = r#"{"message":"message_sent","id":"00000000-0000-0000-0000-00000000007b","source":"00000000-0000-0000-0000-000000000003","content":"Hello, world!","scope":"global"}"#;
        let deserialized: ChatEvent =
            serde_json::from_str(json_data).expect("Deserialization failed");
        if let ChatEvent::MessageSent(message_sent) = deserialized {
            assert_eq!(message_sent.id, MessageId::from_u128(123));
            assert_eq!(message_sent.source, ParticipantId::from_u128(3));
            assert_eq!(message_sent.content, "Hello, world!");
            assert_eq!(message_sent.scope, Scope::Global);
        } else {
            panic!("Expected ChatEvent::MessageSent");
        }
    }

    #[test]
    fn serialize_history_cleared() {
        let event = ChatEvent::HistoryCleared(HistoryCleared {
            issued_by: ParticipantId::from_u128(4),
        });
        let serialized = serde_json::to_string(&event).expect("Serialization failed");
        assert_eq!(
            serialized,
            r#"{"message":"history_cleared","issued_by":"00000000-0000-0000-0000-000000000004"}"#
        );
    }

    #[test]
    fn deserialize_history_cleared() {
        let json_data =
            r#"{"message":"history_cleared","issued_by":"00000000-0000-0000-0000-000000000004"}"#;
        let deserialized: ChatEvent =
            serde_json::from_str(json_data).expect("Deserialization failed");
        if let ChatEvent::HistoryCleared(history_cleared) = deserialized {
            assert_eq!(history_cleared.issued_by, ParticipantId::from_u128(4));
        } else {
            panic!("Expected ChatEvent::HistoryCleared");
        }
    }

    #[test]
    fn serialize_set_last_seen_timestamp() {
        let event = ChatEvent::SetLastSeenTimestamp(SetLastSeenTimestamp {
            scope: Scope::Global,
            timestamp: "2025-05-15T11:40:45.115768379Z".parse().unwrap(),
        });
        let serialized = serde_json::to_string(&event).expect("Serialization failed");
        assert_eq!(
            serialized,
            r#"{"message":"set_last_seen_timestamp","scope":"global","timestamp":"2025-05-15T11:40:45.115768379Z"}"#
        );
    }

    #[test]
    fn deserialize_set_last_seen_timestamp() {
        let json_data = r#"{"message":"set_last_seen_timestamp","scope":"global", "timestamp": "2025-05-15T11:40:45.115768379Z"}"#;
        let deserialized: ChatEvent =
            serde_json::from_str(json_data).expect("Deserialization failed");
        if let ChatEvent::SetLastSeenTimestamp(set_last_seen_timestamp) = deserialized {
            assert_eq!(set_last_seen_timestamp.scope, Scope::Global);
        } else {
            panic!("Expected ChatEvent::HistoryCleared");
        }
    }

    #[test]
    fn serialize_chat_error() {
        let event = ChatEvent::Error(ChatError::ChatDisabled);
        let serialized = serde_json::to_string(&event).expect("Serialization failed");
        assert_eq!(serialized, r#"{"message":"error","error":"chat_disabled"}"#);
    }

    #[test]
    fn deserialize_chat_error() {
        let json_data = r#"{"message":"error","error":"chat_disabled"}"#;
        let deserialized: ChatEvent =
            serde_json::from_str(json_data).expect("Deserialization failed");
        if let ChatEvent::Error(ChatError::ChatDisabled) = deserialized {
            // Success
        } else {
            panic!("Expected ChatEvent::Error(ChatError::ChatDisabled)");
        }
    }
}
