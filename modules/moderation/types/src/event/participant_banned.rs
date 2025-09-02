// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_signaling::banned_participant::BannedParticipant;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

/// Received by moderators on join or when a participant gets banned
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BannedParticipantInfo {
    /// The participant that got banned
    pub participant_id: ParticipantId,

    #[serde(flatten)]
    pub banned_participant: BannedParticipant,
}

impl From<(&ParticipantId, &BannedParticipant)> for BannedParticipantInfo {
    fn from((participant_id, banned_participant): (&ParticipantId, &BannedParticipant)) -> Self {
        Self {
            participant_id: *participant_id,
            banned_participant: banned_participant.clone(),
        }
    }
}
