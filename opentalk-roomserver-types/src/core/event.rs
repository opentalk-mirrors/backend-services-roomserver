// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::BTreeMap;

use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::{
    connection_id::ConnectionId, disconnect_reason::DisconnectReason,
    join::join_success::JoinSuccess, shared_json::SharedJson,
    signaling::module_error::SignalingModuleError,
};

/// Outgoing websocket messages in the core namespace
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoreEvent {
    /// Message sent to a participant on a successful join
    JoinSuccess(Box<JoinSuccess>),

    /// Broadcast message sent to all participants when a new participant has joined
    ParticipantConnected {
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        peer_data: BTreeMap<ModuleId, SharedJson>,
    },

    /// Broadcast message sent to all participants when a participant disconnected
    ParticipantDisconnected {
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        reason: DisconnectReason,
    },

    /// Sent to the moderator when a participant joined the waiting room
    JoinedWaitingRoom { id: ParticipantId },

    /// Sent to the moderators when a participant left the waiting room
    LeftWaitingRoom(LeftWaitingRoom),

    /// Sent to participants who are placed into a waiting room
    InWaitingRoom {
        connection_id: ConnectionId,
        participant_id: ParticipantId,
    },

    /// An error happened when executing a `core` command
    Error(CoreError),
}

/// A participant left the waiting room
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LeftWaitingRoom {
    /// The participant id for the associated participant
    pub id: ParticipantId,
    /// The connection id for the associated participant
    pub connection_id: ConnectionId,
}

impl From<CoreError> for CoreEvent {
    fn from(value: CoreError) -> Self {
        Self::Error(value)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CoreError {
    /// The requested participant is not connected
    UnknownParticipant,
    /// The participant cannot enter the room because they were not accepted by a moderator yet.
    NotAccepted,
}

impl From<CoreError> for SignalingModuleError<CoreError> {
    fn from(value: CoreError) -> Self {
        Self::Module(value)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_types_signaling::ParticipantId;

    use super::{CoreEvent, LeftWaitingRoom};
    use crate::connection_id::ConnectionId;

    #[test]
    fn joined_waiting_room() {
        let produced = serde_json::to_string_pretty(&CoreEvent::JoinedWaitingRoom {
            id: ParticipantId::from_u128(123),
        })
        .unwrap();

        assert_snapshot!(produced, @r#"
        {
          "joined_waiting_room": {
            "id": "00000000-0000-0000-0000-00000000007b"
          }
        }
        "#);
    }

    #[test]
    fn left_waiting_room() {
        let left_waiting_room = LeftWaitingRoom {
            id: opentalk_types_signaling::ParticipantId::from_u128(456),
            connection_id: ConnectionId::from_u128(567),
        };

        let produced =
            serde_json::to_string_pretty(&CoreEvent::LeftWaitingRoom(left_waiting_room.clone()))
                .unwrap();

        assert_snapshot!(produced, @r#"
        {
          "left_waiting_room": {
            "id": "00000000-0000-0000-0000-0000000001c8",
            "connection_id": "00000000-0000-0000-0000-000000000237"
          }
        }
        "#);
    }
}
