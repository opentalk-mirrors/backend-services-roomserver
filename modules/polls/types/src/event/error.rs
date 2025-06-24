// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types::signaling::module_error::ModuleError;
use serde::{Deserialize, Serialize};

/// Errors from the `polls` module namespace
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "error")]
pub enum Error {
    /// Attempted to perform a command which requires more permissions
    InsufficientPermissions,

    /// Attempted to start a poll with invalid choice count
    InvalidChoiceCount {
        /// The minimum number of choices a poll must have
        min_choice_count: usize,
        /// The maximum number of choices a poll is allowed to have
        max_choice_count: usize,
    },

    /// Attempted to perform a command with an invalid poll id
    InvalidPollId,

    /// Attempted to perform a command with an invalid choice id
    InvalidChoiceId,

    /// Attempted to vote for multiple choices although this is not allowed
    MultipleChoicesNotAllowed,

    /// Attempted to perform a command with an invalid choice description
    InvalidChoiceDescriptionLength {
        /// The minimum number of bytes a choice description must have
        min_length: usize,
        /// The maximum number of bytes a choice description is allowed to have
        max_length: usize,
    },

    /// Attempted to perform a command with an invalid duration
    InvalidDuration {
        /// The maximum allowed duration of a poll
        max_duration: u64,
    },

    /// Attempted to perform a command with an invalid topic length
    InvalidTopicLength {
        /// The maximum number of bytes the topic length is allowed to have
        max_length: usize,
    },

    /// Attempted to start a new poll while an existing one is still running
    StillRunning,

    /// An internal error occured and the poll was cancelled
    Internal,
}

impl ModuleError for Error {}
