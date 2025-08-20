// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use super::WhisperGroupOutgoing;

/// An invite to a whisper group
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhisperInvite {
    /// The issuer of the invite
    pub issuer: ParticipantId,
    /// The whisper group
    #[serde(flatten)]
    pub group: WhisperGroupOutgoing,
}
