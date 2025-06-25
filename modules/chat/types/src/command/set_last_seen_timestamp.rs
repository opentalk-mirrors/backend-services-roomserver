// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::time::Timestamp;

use crate::Scope;

/// Set the last seen timestamp for a specific scope
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SetLastSeenTimestamp {
    /// Scope of the timestamp
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub scope: Scope,

    /// Last seen timestamp
    pub timestamp: Timestamp,
}
