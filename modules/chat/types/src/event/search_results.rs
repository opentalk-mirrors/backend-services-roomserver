// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use crate::{Scope, state::ChatChunk};

/// Results from a search in the chat history
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchResults {
    /// A chunk of messages matching the search term
    pub matches: ChatChunk,
    /// The [`Scope`] of the messages
    #[serde(flatten)]
    pub scope: Scope,
}
