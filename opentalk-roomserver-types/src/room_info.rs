// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to the `JoinSuccess` message in the `control` namespace

use opentalk_types_common::{
    rooms::{RoomId, RoomPassword},
    users::UserInfo,
};

/// Information about a room
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RoomInfo {
    /// The id of the room
    pub id: RoomId,

    /// The room password
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<RoomPassword>,

    /// The room creator
    pub created_by: UserInfo,
}
