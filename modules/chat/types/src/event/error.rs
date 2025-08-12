// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types::signaling::module_error::ModuleError;
use serde::{Deserialize, Serialize};

/// Errors from the `chat` module namespace
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum Error {
    /// Request while chat is disabled
    ChatDisabled,

    /// Request user has insufficient permissions
    InsufficientPermissions,

    /// The provided participant ID is unknown
    UnknownParticipant,

    /// The chat messages breakout scope does not match the participants breakout room
    InvalidBreakoutScope,

    /// The provided search term has an invalid length
    InvalidSearchTermLength {
        /// The minimum length a search term must have
        min: usize,
    },
}

impl ModuleError for Error {}
