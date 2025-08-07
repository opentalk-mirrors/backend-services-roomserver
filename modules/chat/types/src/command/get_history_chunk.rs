// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use crate::Scope;

/// Gets a chunk of the message history in the specified `scope`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetHistoryChunk {
    /// Determines which [`ChatChunk`](crate::state::ChatChunk) is requested.
    /// This is always the newest message of the chunk.
    pub message_index: u64,

    /// The scope of the chat history
    #[serde(flatten)]
    pub scope: Scope,
}
