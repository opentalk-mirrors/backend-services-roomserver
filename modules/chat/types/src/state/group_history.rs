// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::users::GroupName;

use crate::state::StoredMessage;

/// Group chat history
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GroupHistory {
    /// Name of the group
    pub name: GroupName,

    /// Group chat history
    pub history: Vec<StoredMessage>,
}
