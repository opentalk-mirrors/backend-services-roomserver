// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::BTreeSet;

use opentalk_types_common::{modules::ModuleId, time::Timestamp};
use opentalk_types_signaling::{ParticipantId, SignalingModuleFrontendData};
use serde::{Deserialize, Serialize};

use crate::RAISE_HANDS_MODULE_ID;

/// The state of the `raise-hands` module.
///
/// This struct is sent to the participant in the `join_success` message
/// when they join successfully to the meeting.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaiseHandsState {
    /// Is raise hands enabled
    pub raise_hands_enabled: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub raised_hands: Option<BTreeSet<RaisedHandState>>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RaisedHandState {
    pub participant_id: ParticipantId,
    pub raised_at: Timestamp,
}

impl SignalingModuleFrontendData for RaiseHandsState {
    const NAMESPACE: Option<ModuleId> = Some(RAISE_HANDS_MODULE_ID);
}
