// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::command::{SendMessage, SetLastSeenTimestamp};

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

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use opentalk_types_common::users::GroupName;
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::Scope;

    #[test]
    fn user_private_message() {
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

    #[test]
    fn user_group_message() {
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
    fn user_room_message() {
        let json = json!({
            "action": "send_message",
            "scope": "global",
            "content": "Hello all!"
        });

        let msg: ChatCommand = serde_json::from_value(json).unwrap();

        if let ChatCommand::SendMessage(SendMessage { content, scope }) = msg {
            assert_eq!(scope, Scope::Global);
            assert_eq!(content, "Hello all!");
        } else {
            panic!()
        }
    }
}
