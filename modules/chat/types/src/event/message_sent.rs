// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::{MessageId, Scope};

/// A message was sent
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageSent {
    /// Id of the message
    pub id: MessageId,

    /// Sender of the message
    pub source: ParticipantId,

    /// Content of the message
    pub content: String,

    /// Scope of the message
    #[serde(flatten)]
    pub scope: Scope,
}
