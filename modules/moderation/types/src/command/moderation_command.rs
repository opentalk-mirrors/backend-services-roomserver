// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `moderation` namespace

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use serde::{Deserialize, Serialize};

use crate::{KickScope, command::Kick, event::ModerationEvent};

/// Commands for the `moderation` namespace
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ModerationCommand {
    /// Kick a participant from the room
    Kick(Kick),

    /// Start the debriefing
    Debrief(KickScope),
}

impl From<Kick> for ModerationCommand {
    fn from(value: Kick) -> Self {
        Self::Kick(value)
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
}
