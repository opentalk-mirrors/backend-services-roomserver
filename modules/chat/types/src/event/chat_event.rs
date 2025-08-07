// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `chat` namespace

use serde::{Deserialize, Serialize};

use crate::{
    command::SetLastSeenTimestamp,
    event::{ChatDisabled, ChatEnabled, Error, HistoryCleared, MessageSent},
    state::{BreakoutHistory, ChatChunk, GroupHistory, PrivateHistory},
};

/// A chat event which occurred
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum ChatEvent {
    /// Chat event where chat was enabled see [ChatEnabled]
    ChatEnabled(ChatEnabled),

    /// Chat event where chat was disabled see [ChatDisabled]
    ChatDisabled(ChatDisabled),

    /// A chunk of the rooms chat history
    RoomChatHistoryChunk {
        /// Room chat history chunk
        history: ChatChunk,
    },

    /// A chunk of a groups chat history
    GroupChatHistoryChunk(GroupHistory),

    /// A chunk of a breakout rooms chat history
    BreakoutChatHistoryChunk(BreakoutHistory),

    /// A chunk of a private chat history between two participants
    PrivateChatHistoryChunk(PrivateHistory),

    /// Chat event where a message was sent see [MessageSent]
    MessageSent(MessageSent),

    /// Chat event where history was cleared see [HistoryCleared]
    HistoryCleared(HistoryCleared),

    /// Chat event when last seen timestamp was set.
    SetLastSeenTimestamp(SetLastSeenTimestamp),

    /// Chat event which errored see [Error]
    Error(Error),
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

impl From<Error> for ChatEvent {
    fn from(value: Error) -> Self {
        Self::Error(value)
    }
}

impl From<SetLastSeenTimestamp> for ChatEvent {
    fn from(value: SetLastSeenTimestamp) -> Self {
        Self::SetLastSeenTimestamp(value)
    }
}

#[cfg(test)]
mod serde_tests {
    use std::str::FromStr;

    use insta::assert_snapshot;
    use opentalk_types_common::{time::Timestamp, users::GroupName};
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::{MessageId, Scope};

    #[test]
    fn serialize_chat_enabled() {
        let event = ChatEvent::ChatEnabled(ChatEnabled {
            issued_by: ParticipantId::from_u128(1),
        });
        let serialized = serde_json::to_value(&event).expect("Serialization failed");
        assert_eq!(
            serialized,
            json!({
                "message":"chat_enabled",
                "issued_by":"00000000-0000-0000-0000-000000000001"
            })
        );
    }

    #[test]
    fn deserialize_chat_enabled() {
        let json_data = json!(
            {
                "message":"chat_enabled",
                "issued_by":"00000000-0000-0000-0000-000000000001"
            }
        );
        let deserialized: ChatEvent =
            serde_json::from_value(json_data).expect("Deserialization failed");
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
        let serialized = serde_json::to_value(&event).expect("Serialization failed");
        assert_eq!(
            serialized,
            json!(
                {
                    "message":"chat_disabled",
                    "issued_by":"00000000-0000-0000-0000-000000000002"
                }
            )
        );
    }

    #[test]
    fn deserialize_chat_disabled() {
        let json_data = json!(
            {
                "message":"chat_disabled",
                "issued_by":"00000000-0000-0000-0000-000000000002"
            }
        );

        let deserialized: ChatEvent =
            serde_json::from_value(json_data).expect("Deserialization failed");
        if let ChatEvent::ChatDisabled(chat_disabled) = deserialized {
            assert_eq!(chat_disabled.issued_by, ParticipantId::from_u128(2));
        } else {
            panic!("Expected ChatEvent::ChatDisabled");
        }
    }

    #[test]
    fn serialize_history_cleared() {
        let event = ChatEvent::HistoryCleared(HistoryCleared {
            issued_by: ParticipantId::from_u128(4),
        });
        let serialized = serde_json::to_value(&event).expect("Serialization failed");
        assert_eq!(
            serialized,
            json!({
                "message":"history_cleared",
                "issued_by":"00000000-0000-0000-0000-000000000004"
            })
        );
    }

    #[test]
    fn deserialize_history_cleared() {
        let json_data = json!(
            {
                "message":"history_cleared",
                "issued_by":"00000000-0000-0000-0000-000000000004"
            }
        );

        let deserialized: ChatEvent =
            serde_json::from_value(json_data).expect("Deserialization failed");
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
            timestamp: Timestamp::unix_epoch(),
        });
        let serialized = serde_json::to_value(&event).expect("Serialization failed");
        assert_eq!(
            serialized,
            json!({
                "message":"set_last_seen_timestamp",
                "scope":"global",
                "timestamp":"1970-01-01T00:00:00Z"
            })
        );
    }

    #[test]
    fn deserialize_set_last_seen_timestamp() {
        let json_data = json!(
            {
                "message":"set_last_seen_timestamp",
                "scope":"global",
                "timestamp": "1970-01-01T00:00:00Z"
            }
        );

        let deserialized: ChatEvent =
            serde_json::from_value(json_data).expect("Deserialization failed");
        if let ChatEvent::SetLastSeenTimestamp(set_last_seen_timestamp) = deserialized {
            assert_eq!(set_last_seen_timestamp.scope, Scope::Global);
        } else {
            panic!("Expected ChatEvent::HistoryCleared");
        }
    }

    #[test]
    fn serialize_global_message() {
        let produced = serde_json::to_value(ChatEvent::MessageSent(MessageSent {
            id: MessageId::nil(),
            source: ParticipantId::nil(),
            content: "Hello All!".to_string(),
            scope: Scope::Global,
        }))
        .unwrap();

        let expected = json!({
            "message": "message_sent",
            "id": "00000000-0000-0000-0000-000000000000",
            "source": "00000000-0000-0000-0000-000000000000",
            "content": "Hello All!",
            "scope": "global"
        });

        assert_eq!(expected, produced);
    }

    #[test]
    fn deserialize_global_message() {
        let json_data = json!(
            {
                "message":"message_sent",
                "id":"00000000-0000-0000-0000-00000000007b",
                "source":"00000000-0000-0000-0000-000000000003",
                "content":"Hello, world!",
                "scope":"global"
            }
        );

        let deserialized: ChatEvent =
            serde_json::from_value(json_data).expect("Deserialization failed");
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
    fn serialize_group_message() {
        let produced = serde_json::to_value(ChatEvent::MessageSent(MessageSent {
            id: MessageId::nil(),
            source: ParticipantId::nil(),
            content: "Hello managers!".to_string(),
            scope: Scope::Group(GroupName::from("management".to_owned())),
        }))
        .unwrap();
        let expected = json!({
            "message":"message_sent",
            "id":"00000000-0000-0000-0000-000000000000",
            "source":"00000000-0000-0000-0000-000000000000",
            "content":"Hello managers!",
            "scope":"group",
            "target":"management",
        });
        assert_eq!(expected, produced);
    }

    #[test]
    fn deserialize_group_message() {
        let json_data = json!(
            {
                "message":"message_sent",
                "id":"00000000-0000-0000-0000-00000000007b",
                "source":"00000000-0000-0000-0000-000000000003",
                "content":"Hello, world!",
                "scope":"group",
                "target":"test"
            }
        );

        let deserialized: ChatEvent =
            serde_json::from_value(json_data).expect("Deserialization failed");
        if let ChatEvent::MessageSent(message_sent) = deserialized {
            assert_eq!(message_sent.id, MessageId::from_u128(123));
            assert_eq!(message_sent.source, ParticipantId::from_u128(3));
            assert_eq!(message_sent.content, "Hello, world!");
            assert_eq!(
                message_sent.scope,
                Scope::Group("test".parse().expect("Valid group name"))
            );
        } else {
            panic!("Expected ChatEvent::MessageSent");
        }
    }

    #[test]
    fn serialize_private_message() {
        let produced = serde_json::to_value(ChatEvent::MessageSent(MessageSent {
            id: MessageId::nil(),
            source: ParticipantId::nil(),
            content: "Hello All!".to_string(),
            scope: Scope::Private(ParticipantId::from_u128(1)),
        }))
        .unwrap();

        let expected = json!({
            "message": "message_sent",
            "id": "00000000-0000-0000-0000-000000000000",
            "source": "00000000-0000-0000-0000-000000000000",
            "content": "Hello All!",
            "scope": "private",
            "target": "00000000-0000-0000-0000-000000000001",
        });
        assert_eq!(expected, produced);
    }

    #[test]
    fn deserialize_private_message() {
        let json_data = json!(
            {
                "message":"message_sent",
                "id":"00000000-0000-0000-0000-00000000007b",
                "source":"00000000-0000-0000-0000-000000000003",
                "content":"Hello, world!",
                "scope":"private",
                "target":"00000000-0000-0000-0000-000000000001"
            }
        );

        let deserialized: ChatEvent =
            serde_json::from_value(json_data).expect("Deserialization failed");
        if let ChatEvent::MessageSent(message_sent) = deserialized {
            assert_eq!(message_sent.id, MessageId::from_u128(123));
            assert_eq!(message_sent.source, ParticipantId::from_u128(3));
            assert_eq!(message_sent.content, "Hello, world!");
            assert_eq!(
                message_sent.scope,
                Scope::Private(ParticipantId::from_u128(1))
            );
        } else {
            panic!("Expected ChatEvent::MessageSent");
        }
    }

    #[test]
    fn serialize_error() {
        let produced = serde_json::to_value(ChatEvent::Error(Error::ChatDisabled)).unwrap();

        let expected = json!({
            "message": "error",
            "error": "chat_disabled",
        });
        assert_eq!(expected, produced);
    }

    #[test]
    fn deserialize_error() {
        let json_data = json!(
            {
                "message":"error",
                "error":"insufficient_permissions"
            }
        );

        let deserialized: ChatEvent =
            serde_json::from_value(json_data).expect("Deserialization failed");
        if let ChatEvent::Error(Error::InsufficientPermissions) = deserialized {
            // Success
        } else {
            panic!("Expected ChatEvent::Error(ChatError::InsufficientPermissions)");
        }
    }

    #[test]
    fn serialize_room_chat_history_chunk() {
        let produced = serde_json::to_string_pretty(&ChatEvent::RoomChatHistoryChunk {
            history: ChatChunk::default(),
        })
        .expect("Serialization failed");

        assert_snapshot!(produced, @r#"
        {
          "message": "room_chat_history_chunk",
          "history": {
            "messages": [],
            "next_index": null
          }
        }
        "#);
    }

    #[test]
    fn deserialize_room_chat_history_chunk() {
        let json = json!({
          "message": "room_chat_history_chunk",
          "history": {
            "messages": [],
            "next_index": null
          }
        });
        let produced = serde_json::from_value(json).expect("Deserialization failed");

        let expected = ChatEvent::RoomChatHistoryChunk {
            history: ChatChunk::default(),
        };

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_group_chat_history_chunk() {
        let produced =
            serde_json::to_string_pretty(&ChatEvent::GroupChatHistoryChunk(GroupHistory {
                name: GroupName::from_str("group1").unwrap(),
                history: ChatChunk::default(),
            }))
            .expect("Serialization failed");

        assert_snapshot!(produced, @r#"
        {
          "message": "group_chat_history_chunk",
          "name": "group1",
          "history": {
            "messages": [],
            "next_index": null
          }
        }
        "#);
    }

    #[test]
    fn deserialize_group_chat_history_chunk() {
        let json = json!(        {
          "message": "group_chat_history_chunk",
          "name": "group1",
          "history": {
            "messages": [],
            "next_index": null
          }
        });
        let produced = serde_json::from_value(json).expect("Deserialization failed");

        let expected = ChatEvent::GroupChatHistoryChunk(GroupHistory {
            name: GroupName::from_str("group1").unwrap(),
            history: ChatChunk::default(),
        });

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_private_chat_history_chunk() {
        let produced =
            serde_json::to_string_pretty(&ChatEvent::PrivateChatHistoryChunk(PrivateHistory {
                correspondent: ParticipantId::nil(),
                history: ChatChunk::default(),
            }))
            .expect("Serialization failed");

        assert_snapshot!(produced, @r#"
        {
          "message": "private_chat_history_chunk",
          "correspondent": "00000000-0000-0000-0000-000000000000",
          "history": {
            "messages": [],
            "next_index": null
          }
        }
        "#);
    }

    #[test]
    fn deserialize_private_chat_history_chunk() {
        let json = json!(        {
          "message": "private_chat_history_chunk",
          "correspondent": "00000000-0000-0000-0000-000000000000",
          "history": {
            "messages": [],
            "next_index": null
          }
        });
        let produced = serde_json::from_value(json).expect("Deserialization failed");

        let expected = ChatEvent::PrivateChatHistoryChunk(PrivateHistory {
            correspondent: ParticipantId::nil(),
            history: ChatChunk::default(),
        });

        assert_eq!(expected, produced);
    }
}
