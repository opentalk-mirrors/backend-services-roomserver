// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use connection_info::ConnectionInfo;
use event_info::EventInfo;
use opentalk_roomserver_types::{connection_id::ConnectionId, device_id::DeviceId};
use opentalk_types_common::{
    events::MeetingDetails, tariffs::TariffResource, time::Timestamp, users::DisplayName,
};
use opentalk_types_signaling::{ParticipantId, Role};
use opentalk_types_signaling_control::room::RoomInfo;
use participant::Participant;
use serde::{Deserialize, Serialize};

pub mod connection_info;
pub mod event_info;
pub mod participant;

/// The data received by a participant upon successfully joining a meeting
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JoinSuccess {
    /// The id of the participant who joined
    pub id: ParticipantId,

    /// The connection id of the participant who joined
    pub connection_id: ConnectionId,

    /// The device id of the participant who joined
    pub device_id: DeviceId,

    /// The other active connections and devices of the participant who joined
    pub connections: Vec<ConnectionInfo>,

    /// The display name of the participant who joined
    pub display_name: DisplayName,

    /// The URL to the avatar of the participant who joined
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,

    /// The role of the participant in the meeting
    pub role: Role,

    /// The timestamp when the meeting will close
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closes_at: Option<Timestamp>,

    /// The tariff of the meeting
    pub tariff: Box<TariffResource>,

    /// The module data for the participant
    pub module_data: opentalk_types_signaling::ModuleData,

    /// List of participants in the meeting
    pub participants: Vec<Participant>,

    /// Information about the event which is associated with the room
    #[serde(default)]
    pub event_info: Option<EventInfo>,

    /// Information about the meeting
    pub meeting_details: MeetingDetails,

    /// Information about the current room
    pub room_info: RoomInfo,

    /// Flag indicating if the participant is the room owner
    #[serde(default)]
    pub is_room_owner: bool,
}

impl JoinSuccess {
    /// Gets the inner module of a JoinSuccess Message
    pub fn get_module<T: opentalk_types_signaling::SignalingModuleFrontendData>(
        &self,
    ) -> Result<Option<T>, serde_json::Error> {
        self.module_data.get()
    }
}
