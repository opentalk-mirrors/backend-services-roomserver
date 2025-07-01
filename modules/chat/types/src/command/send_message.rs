// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use crate::Scope;

/// Send a chat message content with a specific scope
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SendMessage {
    /// The content of the message
    pub content: String,

    /// The scope of the message
    #[serde(flatten)]
    pub scope: Scope,
}
