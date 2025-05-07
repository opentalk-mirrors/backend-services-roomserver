// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::BTreeSet;

use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

pub enum MessageTarget {
    AllParticipantsInRoom,
    Participant(ParticipantId),
    Participants(BTreeSet<ParticipantId>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalingCommand {
    pub namespace: ModuleId,
    pub transaction_id: Option<u64>,
    pub content: Box<serde_json::value::RawValue>,
}

impl SignalingCommand {
    pub fn new(
        namespace: ModuleId,
        transaction_id: Option<u64>,
        content: Box<serde_json::value::RawValue>,
    ) -> Self {
        Self {
            namespace,
            transaction_id,
            content,
        }
    }
}
