// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::{command::ParticipantTargets, whisper_id::WhisperId};

/// Another set of participants was invited to the whisper group
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParticipantsInvited {
    /// The id of the whisper group
    pub whisper_id: WhisperId,
    /// The participants that were invited
    pub participant_ids: Vec<ParticipantId>,
}

impl From<ParticipantTargets> for ParticipantsInvited {
    fn from(value: ParticipantTargets) -> Self {
        Self {
            whisper_id: value.whisper_id,
            participant_ids: value.participant_ids.iter().copied().collect(),
        }
    }
}
