// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

/// Update the ready status
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdatedReadyStatus {
    /// The participant that updated its status
    pub participant_id: ParticipantId,
    /// The new status
    pub status: bool,
}
