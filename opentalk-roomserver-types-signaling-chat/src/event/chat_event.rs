// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `chat` namespace

use crate::event::{ChatDisabled, ChatEnabled, Error, HistoryCleared, MessageSent};

/// A chat event which occurred
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "message", rename_all = "snake_case")
)]
pub enum ChatEvent {
    /// Chat event where chat was enabled see [ChatEnabled]
    ChatEnabled(ChatEnabled),

    /// Chat event where chat was disabled see [ChatDisabled]
    ChatDisabled(ChatDisabled),

    /// Chat event where a message was sent see [MessageSent]
    MessageSent(MessageSent),

    /// Chat event where history was cleared see [HistoryCleared]
    HistoryCleared(HistoryCleared),

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

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use opentalk_types_common::users::GroupName;
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::{MessageId, Scope};

    #[test]
    fn global_serialize() {
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
    fn group_serialize() {
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
    fn private_serialize() {
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
    fn error_serialize() {
        let produced = serde_json::to_value(ChatEvent::Error(Error::ChatDisabled)).unwrap();
        let expected = json!({
            "message": "error",
            "error": "chat_disabled",
        });
        assert_eq!(expected, produced);
    }
}
