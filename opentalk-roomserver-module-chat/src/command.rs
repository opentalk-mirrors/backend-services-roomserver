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
