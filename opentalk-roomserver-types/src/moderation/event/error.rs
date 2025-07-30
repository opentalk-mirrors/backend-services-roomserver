// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Error from the `moderation` module namespace
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
