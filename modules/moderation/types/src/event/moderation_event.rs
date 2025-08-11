// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use crate::event::{DebriefingStarted, DisplayNameChanged, ModerationError};

/// Events sent out by the `moderation` module
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum ModerationEvent {
    /// Sent to a participant when they are kicked from a meeting
    Kicked,

    /// Sent out when debriefing of a session started
    DebriefingStarted(DebriefingStarted),

    /// Sent out when the waiting room is enabled
    WaitingRoomEnabled,

    /// Sent out when the waiting room is disabled
    WaitingRoomDisabled,

    /// Sent to a participant when they are accepted by the moderator from the waiting room
    Accepted,

    /// Sent to all participants when a participants display name gets changed
    DisplayNameChanged(DisplayNameChanged),

    /// An error happened when executing a `moderation` command
    Error(ModerationError),
}

impl From<DebriefingStarted> for ModerationEvent {
    fn from(value: DebriefingStarted) -> Self {
        Self::DebriefingStarted(value)
    }
}

impl From<DisplayNameChanged> for ModerationEvent {
    fn from(value: DisplayNameChanged) -> Self {
        Self::DisplayNameChanged(value)
    }
}

impl From<ModerationError> for ModerationEvent {
    fn from(value: ModerationError) -> Self {
        Self::Error(value)
    }
}

#[cfg(test)]
mod serde_tests {
    use insta::assert_snapshot;
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_kicked() {
        let cmd = ModerationEvent::Kicked;

        assert_snapshot!(serde_json::to_string_pretty(&cmd).unwrap(), @r#"
        {
          "message": "kicked"
        }
        "#);
    }

    #[test]
    fn deserialize_kicked() {
        let expected = json!({"message": "kicked"});

        let produced = serde_json::to_value(ModerationEvent::Kicked).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_debriefing_started() {
        let expected = json!({
            "message": "debriefing_started",
            "issued_by": "00000000-0000-0000-0000-000000000000"
        });

        let produced =
            serde_json::to_value(ModerationEvent::DebriefingStarted(DebriefingStarted {
                issued_by: ParticipantId::nil(),
            }))
            .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn deserialize_debriefing_started() {
        let event = ModerationEvent::DebriefingStarted(DebriefingStarted {
            issued_by: ParticipantId::nil(),
        });

        assert_snapshot!(serde_json::to_string_pretty(&event).unwrap(), @r#"
        {
          "message": "debriefing_started",
          "issued_by": "00000000-0000-0000-0000-000000000000"
        }
        "#);
    }

    #[test]
    fn serialize_accepted() {
        let produced = serde_json::to_string_pretty(&ModerationEvent::Accepted).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "accepted"
        }
        "#);
    }

    #[test]
    fn deserialize_accepted() {
        let produced: ModerationEvent = serde_json::from_value(json!({
            "message": "accepted"
        }))
        .unwrap();
        let expected = ModerationEvent::Accepted;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_display_name_changed() {
        let produced = serde_json::to_string_pretty(&ModerationEvent::DisplayNameChanged(
            DisplayNameChanged {
                target: ParticipantId::nil(),
                issued_by: ParticipantId::nil(),
                old_name: "Alice".parse().expect("valid display name"),
                new_name: "Bob".parse().expect("valid display name"),
            },
        ))
        .unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "display_name_changed",
          "target": "00000000-0000-0000-0000-000000000000",
          "issued_by": "00000000-0000-0000-0000-000000000000",
          "old_name": "Alice",
          "new_name": "Bob"
        }
        "#);
    }

    #[test]
    fn deserialize_display_name_changed() {
        let produced: ModerationEvent = serde_json::from_value(json!({
            "message": "display_name_changed",
            "target": "00000000-0000-0000-0000-000000000000",
            "issued_by": "00000000-0000-0000-0000-000000000000",
            "old_name": "Alice",
            "new_name": "Bob"
        }))
        .unwrap();
        let expected = ModerationEvent::DisplayNameChanged(DisplayNameChanged {
            target: ParticipantId::nil(),
            issued_by: ParticipantId::nil(),
            old_name: "Alice".parse().expect("valid display name"),
            new_name: "Bob".parse().expect("valid display name"),
        });

        assert_eq!(produced, expected);
    }
}
