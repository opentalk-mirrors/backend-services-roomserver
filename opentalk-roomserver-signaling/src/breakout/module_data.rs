// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_types::{breakout::BreakoutRoom, room_kind::RoomKind};
use opentalk_types_common::{modules::ModuleId, time::Timestamp};
use opentalk_types_signaling::{SignalingModuleFrontendData, SignalingModulePeerFrontendData};
use serde::{Deserialize, Serialize};

/// The module data that is attached to the `JoinSuccess` message
#[derive(Debug, Serialize, Deserialize)]
pub struct BreakoutModuleData {
    /// The current room of the participant
    pub room: RoomKind,

    /// Active breakout rooms
    pub rooms: Vec<BreakoutRoom>,

    /// The expiry for all breakout rooms
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires: Option<Timestamp>,
}

impl SignalingModuleFrontendData for BreakoutModuleData {
    const NAMESPACE: Option<ModuleId> = Some(super::NAMESPACE);
}

/// The peer module data that is attached to the `JoinSuccess` message
#[derive(Debug, Serialize, Deserialize)]
pub struct BreakoutPeerModuleData {
    /// The current room of the participant
    pub room: RoomKind,
}

impl SignalingModulePeerFrontendData for BreakoutPeerModuleData {
    const NAMESPACE: Option<ModuleId> = Some(super::NAMESPACE);
}
