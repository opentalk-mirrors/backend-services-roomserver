// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::users::DisplayName;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

/// Received by all participants when a participant gets their display name changed
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DisplayNameChanged {
    /// The participant that got their display name changed
    pub target: ParticipantId,
    /// The issuer of the display name change
    pub issued_by: ParticipantId,
    /// The old display name
    pub old_name: DisplayName,
    /// The new display name
    pub new_name: DisplayName,
}
