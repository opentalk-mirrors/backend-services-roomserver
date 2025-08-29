// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_roomserver_types::signaling::module_error::ModuleError;
use opentalk_roomserver_types_livekit::MicrophoneRestrictionErrorKind;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

/// Error from the `moderation` module namespace
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum ModerationError {
    /// Cannot change the display name of registered users
    CannotChangeNameOfRegisteredUsers,
    /// Invalid display name
    InvalidDisplayName,
    /// Insufficient permissions to perform a command
    InsufficientPermissions,
    /// The requested participant is not connected
    UnknownParticipant,
    /// The participant is not known.
    ///
    /// The participant might have disconnected before the command was executed.
    UnknownParticipants {
        /// A list of participants that are currently not part of the meeting.
        participants: BTreeSet<ParticipantId>,
    },
    /// The participant is already banned
    AlreadyBanned,
    /// The participant is already unbanned
    AlreadyUnbanned,
    /// Can't ban the room owner
    CannotBanRoomOwner,
    /// Can't ban guests
    CannotBanGuests,
    /// Cannot ban oneself
    CannotBanSelf,
    /// The participant is not in the waiting room
    NotWaiting,
    /// The participant cannot enter the room because they were not accepted by a moderator yet.
    NotAccepted,
    /// Cannot send the room owner to the waiting room
    CannotSendRoomOwnerToWaitingRoom,
    /// The room owner cannot be kicked
    CannotKickRoomOwner,
    /// An internal error occurred
    Internal,
    /// The received command cannot be executed since there is already a conflicting ongoing task.
    ConflictingTask,
    /// The livekit server is not available
    LivekitUnavailable,
}

impl From<MicrophoneRestrictionErrorKind> for ModerationError {
    fn from(err: MicrophoneRestrictionErrorKind) -> Self {
        match err {
            MicrophoneRestrictionErrorKind::ConflictingTask => ModerationError::ConflictingTask,
            MicrophoneRestrictionErrorKind::LivekitUnavailable => {
                ModerationError::LivekitUnavailable
            }
        }
    }
}

impl ModuleError for ModerationError {}
