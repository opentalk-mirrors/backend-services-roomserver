// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types::client_parameters::Role;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

/// A participants role has been updated
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RoleUpdate {
    /// The affected participant
    pub participant_id: ParticipantId,
    /// The participants new role
    pub new_role: Role,
}
