// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;

use crate::{MessageId, Scope};

/// A message was sent
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MessageSent {
    /// Id of the message
    pub id: MessageId,

    /// Sender of the message
    pub source: ParticipantId,

    /// Content of the message
    pub content: String,

    /// Scope of the message
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub scope: Scope,
}
