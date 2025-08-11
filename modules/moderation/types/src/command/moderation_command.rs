// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `moderation` namespace

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::{
    KickScope,
    command::{ChangeDisplayName, Kick},
    event::ModerationEvent,
};

/// Commands for the `moderation` namespace
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ModerationCommand {
    /// Kick a participant from the room
    Kick(Kick),

    /// Start the debriefing
    Debrief(KickScope),

    /// Enable waiting room for the meeting
    EnableWaitingRoom,

    /// Disable waiting room for the meeting
    DisableWaitingRoom,

    /// Change the display name of the targeted guest
    ChangeDisplayName(ChangeDisplayName),

    /// Accept a participant from the waiting room into the meeting
    Accept(Accept),
}

/// Accept a participant into the meeting
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Accept {
    /// The participant to accept into the meeting
    pub target: ParticipantId,
}

impl From<Kick> for ModerationCommand {
    fn from(value: Kick) -> Self {
        Self::Kick(value)
    }
}

impl From<ChangeDisplayName> for ModerationCommand {
    fn from(value: ChangeDisplayName) -> Self {
        Self::ChangeDisplayName(value)
    }
}

impl CreateReplica<ModerationEvent> for ModerationCommand {
    fn replicate(&self) -> Option<ModerationEvent> {
        None
    }
}

#[cfg(test)]
mod serde_tests {
    use insta::assert_snapshot;
    use opentalk_types_common::users::DisplayName;
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_kick() {
        let cmd = ModerationCommand::Kick(Kick {
            target: ParticipantId::nil(),
        });

        assert_snapshot!(serde_json::to_string_pretty(&cmd).unwrap(), @r#"
        {
          "action": "kick",
          "target": "00000000-0000-0000-0000-000000000000"
        }
        "#);
    }

    #[test]
    fn deserialize_kick() {
        let json = json!({
            "action": "kick",
            "target": "00000000-0000-0000-0000-000000000000"
        });

        let produced: ModerationCommand = serde_json::from_value(json).unwrap();
        let expected = ModerationCommand::Kick(Kick {
            target: ParticipantId::nil(),
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_debrief() {
        let cmd = ModerationCommand::Debrief(KickScope::All);

        assert_snapshot!(serde_json::to_string_pretty(&cmd).unwrap(), @r#"
        {
          "action": "debrief",
          "kick_scope": "all"
        }
        "#);
    }

    #[test]
    fn deserialize_debrief() {
        let json = json!({
            "action": "debrief",
            "kick_scope": "users_and_guests"
        });

        let produced: ModerationCommand = serde_json::from_value(json).unwrap();
        let expected = ModerationCommand::Debrief(KickScope::UsersAndGuests);

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_accept() {
        let cmd = ModerationCommand::Accept(Accept {
            target: ParticipantId::nil(),
        });

        assert_snapshot!(serde_json::to_string_pretty(&cmd).unwrap(), @r#"
        {
          "action": "accept",
          "target": "00000000-0000-0000-0000-000000000000"
        }
        "#);
    }

    #[test]
    fn deserialize_accept() {
        let json = json!({
            "action": "accept",
            "target": "00000000-0000-0000-0000-000000000000",
        });
        let produced: ModerationCommand = serde_json::from_value(json).unwrap();
        let expected = ModerationCommand::Accept(Accept {
            target: ParticipantId::nil(),
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_change_display_name() {
        let cmd = ModerationCommand::ChangeDisplayName(ChangeDisplayName {
            new_name: DisplayName::from_str_lossy("Alice"),
            target: ParticipantId::nil(),
        });

        assert_snapshot!(serde_json::to_string_pretty(&cmd).unwrap(), @r#"
        {
          "action": "change_display_name",
          "new_name": "Alice",
          "target": "00000000-0000-0000-0000-000000000000"
        }
        "#);
    }

    #[test]
    fn deserialize_change_display_name() {
        let json = json!({
            "action": "change_display_name",
            "new_name": "Alice",
            "target": "00000000-0000-0000-0000-000000000000"
        });
        let produced: ModerationCommand = serde_json::from_value(json).unwrap();
        let expected = ModerationCommand::ChangeDisplayName(ChangeDisplayName {
            new_name: DisplayName::from_str_lossy("Alice"),
            target: ParticipantId::nil(),
        });

        assert_eq!(produced, expected);
    }
}
