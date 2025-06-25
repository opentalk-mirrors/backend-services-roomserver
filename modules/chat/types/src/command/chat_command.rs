// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_signaling::signaling_module::CreateReplica;

use crate::{
    command::{SendMessage, SetLastSeenTimestamp},
    event::ChatEvent,
};

/// Commands for the `chat` namespace
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "action", rename_all = "snake_case")
)]
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

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use opentalk_types_common::{time::Timestamp, users::GroupName};
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::Scope;

    #[test]
    fn serialize_enable_chat() {
        let command = ChatCommand::EnableChat;
        let serialized = serde_json::to_value(&command).expect("Serialization failed");
        assert_eq!(
            serialized,
            json!(
                {
                    "action":"enable_chat"
                }
            )
        );
    }

    #[test]
    fn deserialize_enable_chat() {
        let json_data = json!(
            {
                "action":"enable_chat"
            }
        );
        let deserialized: ChatCommand =
            serde_json::from_value(json_data).expect("Deserialization failed");
        assert_eq!(deserialized, ChatCommand::EnableChat);
    }

    #[test]
    fn serialize_set_last_seen_timestamp() {
        let set_last_seen = SetLastSeenTimestamp {
            scope: Scope::Global,
            timestamp: Timestamp::unix_epoch(),
        };

        let command = ChatCommand::SetLastSeenTimestamp(set_last_seen.clone());
        let serialized = serde_json::to_value(&command).expect("Serialization failed");
        let expected = json!(
            {
                "action":"set_last_seen_timestamp",
                "scope":"global",
                "timestamp":"1970-01-01T00:00:00Z"
            }
        );
        assert_eq!(serialized, expected);
    }

    #[test]
    fn deserialize_set_last_seen_timestamp() {
        let json_data = json!(
            {
                "action":"set_last_seen_timestamp",
                "scope":"global",
                "timestamp":"1970-01-01T00:00:00Z"
            }
        );
        let deserialized: ChatCommand =
            serde_json::from_value(json_data).expect("Deserialization failed");
        if let ChatCommand::SetLastSeenTimestamp(set_last_seen) = deserialized {
            assert_eq!(set_last_seen.scope, Scope::Global);
        } else {
            panic!("Expected ChatCommand::SetLastSeenTimestamp");
        }
    }

    #[test]
    fn serialize_global_message() {
        let send_message = SendMessage {
            content: "test message".to_string(),
            scope: Scope::Global,
        };
        let command = ChatCommand::SendMessage(send_message.clone());
        let serialized = serde_json::to_value(&command).expect("Serialization failed");
        let expected = json!(
            {
                "action":"send_message",
                "content":"test message",
                "scope":"global"
            }
        );
        assert_eq!(serialized, expected);
    }

    #[test]
    fn deserialize_global_message() {
        let json_data = json!(
            {
                "action":"send_message",
                "scope":"global",
                "content":"Hello!"
            }
        );

        let deserialized: ChatCommand =
            serde_json::from_value(json_data).expect("Deserialization failed");
        if let ChatCommand::SendMessage(send_message) = deserialized {
            assert_eq!(send_message.content, "Hello!");
            assert_eq!(send_message.scope, Scope::Global);
        } else {
            panic!("Expected ChatCommand::SendMessage");
        }
    }

    #[test]
    fn serialize_group_message() {
        let send_message = SendMessage {
            content: "test message".to_string(),
            scope: Scope::Group("test".parse().expect("Valid group name")),
        };
        let command = ChatCommand::SendMessage(send_message.clone());
        let serialized = serde_json::to_value(&command).expect("Serialization failed");
        let expected = json!(
            {
                "action":"send_message",
                "content":"test message",
                "scope":"group",
                "target":"test"
            }
        );
        assert_eq!(serialized, expected);
    }

    #[test]
    fn deserialize_group_message() {
        let json = json!({
            "action": "send_message",
            "scope": "group",
            "target": "management",
            "content": "Hello managers!"
        });

        let msg: ChatCommand = serde_json::from_value(json).unwrap();

        if let ChatCommand::SendMessage(SendMessage { content, scope }) = msg {
            assert_eq!(
                scope,
                Scope::Group(GroupName::from("management".to_owned()))
            );
            assert_eq!(content, "Hello managers!");
        } else {
            panic!()
        }
    }

    #[test]
    fn serialize_private_message() {
        let send_message = SendMessage {
            content: "test message".to_string(),
            scope: Scope::Private(ParticipantId::from_u128(1)),
        };

        let command = ChatCommand::SendMessage(send_message.clone());
        let serialized = serde_json::to_value(&command).expect("Serialization failed");

        let expected = json!(
            {
                "action":"send_message",
                "content":"test message",
                "scope":"private",
                "target":"00000000-0000-0000-0000-000000000001"}
        );

        assert_eq!(serialized, expected);
    }

    #[test]
    fn deserialize_private_message() {
        let json = json!({
            "action": "send_message",
            "scope": "private",
            "target": "00000000-0000-0000-0000-000000000000",
            "content": "Hello Bob!"
        });

        let msg: ChatCommand = serde_json::from_value(json).unwrap();

        if let ChatCommand::SendMessage(SendMessage { content, scope }) = msg {
            assert_eq!(scope, Scope::Private(ParticipantId::nil()));
            assert_eq!(content, "Hello Bob!");
        } else {
            panic!()
        }
    }
}
