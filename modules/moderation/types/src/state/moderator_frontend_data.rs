// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use opentalk_roomserver_signaling::waiting_participant::WaitingParticipant;
use opentalk_roomserver_types::{connection_id::ConnectionId, room_parameters::WaitingRoom};
use opentalk_types_common::users::DisplayName;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::event::BannedParticipantInfo;

/// Moderation module state that is visible only to moderators
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModeratorJoinInfo {
    /// The state of the waiting room
    pub waiting_room: WaitingRoom,

    /// The participants that are currently in the waiting room (if any)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub waiting_room_participants: Vec<WaitingParticipantPeerData>,

    /// The participants that are currently banned from the room
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub banned_participants: Vec<BannedParticipantInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaitingParticipantPeerData {
    /// The id of the participant
    pub participant_id: ParticipantId,

    /// The connection ids of the participant
    pub connections: Vec<ConnectionId>,

    /// Whether the participant was accepted to enter the meeting
    pub accepted: bool,

    /// The time when the participant joined the waiting room
    pub joined_at: DateTime<Utc>,

    /// The participants display name
    pub display_name: DisplayName,

    /// The participants avatar URL
    pub avatar_url: Option<String>,
}

impl From<(&ParticipantId, &WaitingParticipant)> for WaitingParticipantPeerData {
    fn from((&participant_id, state): (&ParticipantId, &WaitingParticipant)) -> Self {
        Self {
            participant_id,
            connections: state.connections.keys().copied().collect(),
            accepted: state.accepted,
            joined_at: state.joined_at,
            display_name: state.kind.display_name().clone(),
            avatar_url: state.kind.avatar_url().map(String::from),
        }
    }
}
