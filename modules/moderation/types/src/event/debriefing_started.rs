// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

/// Sent out when debriefing of a session started
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebriefingStarted {
    /// The moderator who started the debriefing
    pub issued_by: ParticipantId,
}
