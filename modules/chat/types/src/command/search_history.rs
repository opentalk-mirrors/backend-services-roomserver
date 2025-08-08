// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use crate::Scope;

/// Search in the chat history
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchHistory {
    /// The scope to search in
    #[serde(flatten)]
    pub scope: Scope,
    /// The search term
    pub term: String,
    /// The message index of the [`ChatChunk`](crate::state::ChatChunk) in the
    /// search history. Retrieves the latest [`ChatChunk`](crate::state::ChatChunk)
    /// when [`None`].
    pub message_index: Option<u64>,
}
