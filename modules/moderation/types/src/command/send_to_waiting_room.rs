// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

/// Send a participant to the waiting room
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendToWaitingRoom {
    /// The participant to move to the waiting room
    pub target: ParticipantId,
}
