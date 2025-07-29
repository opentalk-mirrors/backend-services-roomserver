// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

/// Kick a participant from the room
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Kick {
    /// The participant to kick from the room
    pub target: ParticipantId,
}
