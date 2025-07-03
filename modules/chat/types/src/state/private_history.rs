// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::state::StoredMessage;

/// Private chat history
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrivateHistory {
    /// Private chat correspondent
    pub correspondent: ParticipantId,

    /// Private chat history
    pub history: Vec<StoredMessage>,
}
