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
