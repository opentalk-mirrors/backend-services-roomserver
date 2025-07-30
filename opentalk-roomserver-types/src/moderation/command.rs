// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `moderation` namespace

use opentalk_types_signaling::ParticipantId;

/// Commands for the `moderation` namespace
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ModerationCommand {
    /// Accept a participant from the waiting room into the meeting
    Accept(Accept),
}

/// Accept a participant into the meeting
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Accept {
    /// The participant to accept into the meeting
    pub target: ParticipantId,
}

#[cfg(test)]
mod serde_tests {
    use opentalk_types_signaling::ParticipantId;
    use serde_json::json;

    use super::*;

    #[test]
    fn accept() {
        let json = json!({
            "action": "accept",
            "target": "00000000-0000-0000-0000-000000000000"
        });

        let msg: ModerationCommand = serde_json::from_value(json).unwrap();

        assert_eq!(
            msg,
            ModerationCommand::Accept(Accept {
                target: ParticipantId::nil()
            }),
        );
    }
}
