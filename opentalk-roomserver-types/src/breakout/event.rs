// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_common::time::Timestamp;
use opentalk_types_signaling::{ModuleData, ParticipantId};
use serde::{Deserialize, Serialize};

use crate::{
    breakout::{BreakoutRoom, breakout_id::BreakoutId},
    signaling::module_error::ModuleError,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "message")]
pub enum BreakoutEvent {
    /// Breakout rooms have started
    Started {
        /// The issuing participant
        started_by: ParticipantId,
        /// The configured breakout rooms
        rooms: Vec<BreakoutRoom>,
        /// Optional breakout expiry. When the breakout rooms expire, all participants are moved back to the main room
        #[serde(skip_serializing_if = "Option::is_none")]
        expires_at: Option<Timestamp>,
        /// The optional assignment for the receiving participant. This assignment is not enforced by the roomserver
        #[serde(skip_serializing_if = "Option::is_none")]
        assignment: Option<BreakoutId>,
    },

    /// A participant switched between the main and/or breakout room
    ///
    /// This event is automatically triggered when breakout rooms are closed
    ParticipantSwitchedRoom {
        /// The participant that moved
        participant_id: ParticipantId,
        /// The old room of the participant. `None` is the main room
        old_breakout_room: Option<BreakoutId>,
        /// The room that the participant moved to. `None` is the main room
        new_breakout_room: Option<BreakoutId>,
    },

    /// The receiver has successfully switched between rooms
    SwitchedRoom {
        /// Module data that was attached by signaling modules
        module_data: ModuleData,
    },

    /// A notice that the breakout rooms will close soon
    CloseNotice {
        /// The participant that issued the close command
        issued_by: ParticipantId,
        /// The time at which the breakout rooms close
        stops_at: Timestamp,
    },

    /// Breakout rooms are in the process of being closed.
    ///
    /// Is received before all participants are moved back to the main room
    Closing {
        /// The participant that issued the close command. Is `None` when the breakout rooms expired.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        issued_by: Option<ParticipantId>,
    },

    /// The breakout rooms have been closed
    Closed,

    /// A breakout error
    Error(BreakoutError),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "error")]
pub enum BreakoutError {
    /// The requesting user has insufficient permission
    InsufficientPermission,
    /// A breakout configuration is already active
    AlreadyActive,
    /// The participant is already in the targeted room
    AlreadyInRoom,
    /// A provided participant id is unknown to the roomserver
    UnknownParticipant { participant_id: ParticipantId },
    /// Invalid selection of assignments when starting the breakout rooms, e.g. a participant is assigned to multiple rooms
    InvalidSelection,
    /// Provided an unknown BreakoutId
    UnknownBreakoutId,
    /// The breakout rooms are inactive
    BreakoutInactive,
}

impl ModuleError for BreakoutError {}

#[cfg(test)]
mod tests {
    use opentalk_types_common::time::Timestamp;
    use opentalk_types_signaling::{ModuleData, ParticipantId};
    use serde_json::json;

    use crate::breakout::{
        BreakoutRoom,
        breakout_id::BreakoutId,
        event::{BreakoutError, BreakoutEvent},
    };

    #[test]
    fn started() {
        let val = BreakoutEvent::Started {
            started_by: ParticipantId::nil(),
            rooms: vec![
                BreakoutRoom {
                    id: BreakoutId::from(0),
                    name: "Room 1".into(),
                },
                BreakoutRoom {
                    id: BreakoutId::from(1),
                    name: "Room 2".into(),
                },
            ],
            expires_at: Some(Timestamp::unix_epoch()),
            assignment: None,
        };

        let json = serde_json::to_value(val).unwrap();

        assert_eq!(
            json,
            json!({
                "message": "started",
                "started_by": "00000000-0000-0000-0000-000000000000",
                "rooms": [
                    {"id": 0, "name": "Room 1"},
                    {"id": 1, "name": "Room 2"}
                ],
                "expires_at": "1970-01-01T00:00:00Z",
            }
            )
        );
    }

    #[test]
    fn participant_switched_room() {
        let val = BreakoutEvent::ParticipantSwitchedRoom {
            participant_id: ParticipantId::nil(),
            old_breakout_room: None,
            new_breakout_room: Some(BreakoutId::from(1)),
        };

        let json = serde_json::to_value(val).unwrap();

        assert_eq!(
            json,
            json!({
                "message": "participant_switched_room",
                "participant_id": "00000000-0000-0000-0000-000000000000",
                "old_breakout_room": null,
                "new_breakout_room": 1
            }
            )
        );
    }

    #[test]
    fn switched_room() {
        let val = BreakoutEvent::SwitchedRoom {
            module_data: ModuleData::new(),
        };

        let json = serde_json::to_value(val).unwrap();

        assert_eq!(
            json,
            json!({
                "message": "switched_room",
                "module_data": {},
            }
            )
        );
    }

    #[test]
    fn close_notice() {
        let val = BreakoutEvent::CloseNotice {
            issued_by: ParticipantId::nil(),
            stops_at: Timestamp::unix_epoch(),
        };

        let json = serde_json::to_value(val).unwrap();

        assert_eq!(
            json,
            json!({
                "message": "close_notice",
                "issued_by": "00000000-0000-0000-0000-000000000000",
                "stops_at": "1970-01-01T00:00:00Z",
            }
            )
        );
    }

    #[test]
    fn closing() {
        let val = BreakoutEvent::Closing {
            issued_by: Some(ParticipantId::nil()),
        };

        let json = serde_json::to_value(val).unwrap();

        assert_eq!(
            json,
            json!({
                "message": "closing",
                "issued_by": "00000000-0000-0000-0000-000000000000",
            }
            )
        );
    }

    #[test]
    fn closing_issued_by() {
        let val = BreakoutEvent::Closing { issued_by: None };

        let json = serde_json::to_value(val).unwrap();

        assert_eq!(
            json,
            json!({
                "message": "closing"
            }
            )
        );
    }

    #[test]
    fn closed() {
        let val = BreakoutEvent::Closed;

        let json = serde_json::to_value(val).unwrap();

        assert_eq!(
            json,
            json!({
                "message": "closed"
            }
            )
        );
    }

    #[test]
    fn error() {
        let val = BreakoutEvent::Error(BreakoutError::AlreadyActive);

        let json = serde_json::to_value(val).unwrap();

        assert_eq!(
            json,
            json!({
                "message": "error",
                "error": "already_active"
            }
            )
        );
    }

    #[test]
    fn unknown_participant() {
        let val = BreakoutEvent::Error(BreakoutError::UnknownParticipant {
            participant_id: ParticipantId::nil(),
        });
        let json = serde_json::to_value(val).unwrap();

        assert_eq!(
            json,
            json!({
                "message": "error",
                "error": "unknown_participant",
                "participant_id": "00000000-0000-0000-0000-000000000000"
            }
            )
        );
    }
}
