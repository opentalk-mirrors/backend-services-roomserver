// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::time::Timestamp;
use serde::{Deserialize, Serialize};

use crate::Scope;

/// Set the last seen timestamp for a specific scope
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]

pub struct SetLastSeenTimestamp {
    /// Scope of the timestamp
    #[serde(flatten)]
    pub scope: Scope,

    /// Last seen timestamp
    pub timestamp: Timestamp,
}
