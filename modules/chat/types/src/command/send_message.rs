// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::Scope;

/// Send a chat message content with a specific scope
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SendMessage {
    /// The content of the message
    pub content: String,

    /// The scope of the message
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub scope: Scope,
}
