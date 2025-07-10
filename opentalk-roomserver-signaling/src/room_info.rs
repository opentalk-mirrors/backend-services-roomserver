// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_types::room_parameters::RoomParameters;
use opentalk_types_common::{rooms::RoomId, time::Timestamp};

#[derive(Debug, Clone)]
pub struct RoomTaskInfo {
    /// the identifier of the room
    pub room_id: RoomId,
    /// The start parameters for the room task
    pub room: RoomParameters,
    /// The time at which the room will close
    pub closes_at: Option<Timestamp>,
}
