// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::whisper_id::WhisperId;

/// The whisper invite was accepted by a participant
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhisperAccepted {
    /// The id of the whisper group
    pub whisper_id: WhisperId,
    /// The participant that accepted the invite
    pub participant_id: ParticipantId,
}
