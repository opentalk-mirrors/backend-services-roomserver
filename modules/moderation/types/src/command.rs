// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `moderation` namespace

use std::collections::BTreeSet;

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use opentalk_roomserver_types::client_parameters::Role;
use opentalk_types_common::users::DisplayName;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::{KickScope, event::ModerationEvent};

/// Commands for the `moderation` namespace
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ModerationCommand {
    /// Kick a participant from the room
    Kick { target: ParticipantId },

    /// Ban a participant from the room
    Ban { target: ParticipantId },

    /// Unban a banned participant
    Unban { target: ParticipantId },

    /// Change the role of a participant
    UpdateRole {
        /// The affected participant
        participant_id: ParticipantId,
        /// The participants new role
        new_role: Role,
    },

    /// Start the debriefing
    Debrief(KickScope),

    /// Enable waiting room for the meeting
    EnableWaitingRoom,

    /// Disable waiting room for the meeting
    DisableWaitingRoom,

    /// Send a participant to the waiting room
    SendToWaitingRoom {
        /// The participant to move to the waiting room
        target: ParticipantId,
    },

    /// Change the display name of the targeted guest
    ChangeDisplayName {
        /// The new display name
        new_name: DisplayName,

        /// The participant that will have their name changed
        target: ParticipantId,
    },

    /// Accept a participant from the waiting room into the meeting
    Accept {
        /// The participant to accept into the meeting
        target: ParticipantId,
    },

    /// Mutes participants
    Mute {
        /// The participants that should get muted
        participants: BTreeSet<ParticipantId>,
    },

    /// Enables the microphone restriction state where only the participants that are part of the
    /// `unrestricted_participants` are allowed to unmute themselves. This will mute all
    ///  participants who are not allowed to unmute themselves, but are currently not muted.
    EnableMicrophoneRestrictions {
        /// Participants that are still allowed to unmute
        unrestricted_participants: BTreeSet<ParticipantId>,
    },

    /// Disable the microphone restriction state which will allow all participants
    /// to unmute their microphone again.
    DisableMicrophoneRestrictions,
}

/// Accept a participant into the meeting
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Accept {
    /// The participant to accept into the meeting
    pub target: ParticipantId,
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
        let cmd = ModerationCommand::Kick {
            target: ParticipantId::nil(),
        };

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
        let expected = ModerationCommand::Kick {
            target: ParticipantId::nil(),
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_ban() {
        let cmd = ModerationCommand::Ban {
            target: ParticipantId::nil(),
        };

        assert_snapshot!(serde_json::to_string_pretty(&cmd).unwrap(), @r#"
        {
          "action": "ban",
          "target": "00000000-0000-0000-0000-000000000000"
        }
        "#);
    }

    #[test]
    fn deserialize_ban() {
        let json = json!({
            "action": "ban",
            "target": "00000000-0000-0000-0000-000000000000"
        });

        let produced: ModerationCommand = serde_json::from_value(json).unwrap();
        let expected = ModerationCommand::Ban {
            target: ParticipantId::nil(),
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_unban() {
        let cmd = ModerationCommand::Unban {
            target: ParticipantId::nil(),
        };

        assert_snapshot!(serde_json::to_string_pretty(&cmd).unwrap(), @r#"
        {
          "action": "unban",
          "target": "00000000-0000-0000-0000-000000000000"
        }
        "#);
    }

    #[test]
    fn deserialize_unban() {
        let json = json!({
            "action": "unban",
            "target": "00000000-0000-0000-0000-000000000000"
        });

        let produced: ModerationCommand = serde_json::from_value(json).unwrap();
        let expected = ModerationCommand::Unban {
            target: ParticipantId::nil(),
        };

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
        let cmd = ModerationCommand::Accept {
            target: ParticipantId::nil(),
        };

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
        let expected = ModerationCommand::Accept {
            target: ParticipantId::nil(),
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_change_display_name() {
        let cmd = ModerationCommand::ChangeDisplayName {
            new_name: DisplayName::from_str_lossy("Alice"),
            target: ParticipantId::nil(),
        };

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
        let expected = ModerationCommand::ChangeDisplayName {
            new_name: DisplayName::from_str_lossy("Alice"),
            target: ParticipantId::nil(),
        };

        assert_eq!(produced, expected);
    }
}
