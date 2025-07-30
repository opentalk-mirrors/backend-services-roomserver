// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types::signaling::module_error::ModuleError;
use serde::{Deserialize, Serialize};

/// Error from the `moderation` module namespace
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum ModerationError {
    /// Insufficient permissions to perform a command
    InsufficientPermissions,
    /// The requested participant is not connected
    UnknownParticipant,
    /// The participant is not in the waiting room
    NotWaiting,
    /// The participant cannot enter the room because they were not accepted by a moderator yet.
    NotAccepted,
}

impl ModuleError for ModerationError {}
