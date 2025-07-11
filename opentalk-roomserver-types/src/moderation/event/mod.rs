// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::{Participant, ParticipantId};

use crate::connection_id::ConnectionId;
pub use crate::moderation::event::error::ModerationError;

mod error;

/// Events sent out by the `moderation` module
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum ModerationEvent {
    /// Sent to participants who are placed into a waiting room
    InWaitingRoom {
        connection_id: ConnectionId,
        participant_id: ParticipantId,
    },

    /// Sent to the moderator when a participant joined the waiting room
    JoinedWaitingRoom(Participant),

    /// Sent to the moderator when a participant left the waiting room
    LeftWaitingRoom(LeftWaitingRoom),

    /// Sent to a participant when they are accepted by the moderator from the waiting room
    Accepted,

    /// An error happened when executing a `moderation` command
    Error(ModerationError),
}

/// A participant left the waiting room
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LeftWaitingRoom {
    /// The participant id for the associated participant
    pub id: ParticipantId,
    /// The connection id for the associated participant
    pub connection_id: ConnectionId,
}

impl From<ModerationError> for ModerationEvent {
    fn from(value: ModerationError) -> Self {
        Self::Error(value)
    }
}

#[cfg(test)]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn joined_waiting_room() {
        let participant = Participant {
            id: opentalk_types_signaling::ParticipantId::from_u128(123),
            module_data: opentalk_types_signaling::ModulePeerData::new(),
        };
        let expected = json!({
            "message": "joined_waiting_room",
            "id": "00000000-0000-0000-0000-00000000007b"
        });

        let produced =
            serde_json::to_value(ModerationEvent::JoinedWaitingRoom(participant.clone())).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn left_waiting_room() {
        let left_waiting_room = LeftWaitingRoom {
            id: opentalk_types_signaling::ParticipantId::from_u128(456),
            connection_id: ConnectionId::from_u128(567),
        };
        let expected = json!({
            "message": "left_waiting_room",
            "id": "00000000-0000-0000-0000-0000000001c8",
            "connection_id": "00000000-0000-0000-0000-000000000237",
        });

        let produced =
            serde_json::to_value(ModerationEvent::LeftWaitingRoom(left_waiting_room.clone()))
                .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn accepted() {
        let expected = json!({"message": "accepted"});

        let produced = serde_json::to_value(ModerationEvent::Accepted).unwrap();

        assert_eq!(expected, produced);
    }
}
