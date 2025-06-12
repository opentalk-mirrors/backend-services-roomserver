// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::{BTreeMap, BTreeSet, HashMap};

use opentalk_roomserver_types::{
    client_parameters::Role, connection_id::ConnectionId, device_id::DeviceId, room_kind::RoomKind,
};
use opentalk_types_common::users::DisplayName;
use opentalk_types_signaling::ParticipantId;

use crate::participant_filter::{ParticipantsFiltered, ParticipantsFilteredMut};

#[derive(Debug, Default)]
pub struct Participants {
    /// Contains all connected and disconnected participants, even those outside of the current breakout scope
    pub all_unfiltered: HashMap<ParticipantId, ParticipantState>,
}

impl Participants {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn connected(&self) -> ParticipantsFiltered {
        ParticipantsFiltered::new(self).connected()
    }

    pub fn disconnected(&self) -> ParticipantsFiltered {
        ParticipantsFiltered::new(self).disconnected()
    }

    pub fn in_room(&self, room: RoomKind) -> ParticipantsFiltered {
        ParticipantsFiltered::new(self).room(room)
    }

    pub fn connections(&self) -> BTreeMap<ParticipantId, BTreeSet<ConnectionId>> {
        self.all_unfiltered
            .iter()
            .map(|(p, state)| (*p, state.connections.keys().copied().collect()))
            .collect()
    }

    pub fn filter(&self) -> ParticipantsFiltered {
        ParticipantsFiltered::new(self)
    }

    pub fn filter_mut(&mut self) -> ParticipantsFilteredMut {
        ParticipantsFilteredMut::new(self)
    }
}

#[derive(Debug)]
pub struct ParticipantState {
    /// The participants display name
    pub display_name: DisplayName,

    /// The breakout room of the participant. Is `None` when in the main room
    pub room: RoomKind,

    /// The kind of the participant
    pub kind: ParticipantKind,

    /// The role that the participant assumes in the meeting.
    pub role: Role,

    /// All connections and their associated device
    pub connections: HashMap<ConnectionId, DeviceId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticipantKind {
    User,
    Guest,
}

impl ParticipantState {
    pub fn new(display_name: DisplayName, kind: ParticipantKind, role: Role) -> Self {
        Self {
            display_name,
            room: RoomKind::Main,
            kind,
            role,
            connections: HashMap::new(),
        }
    }

    pub fn is_connected(&self) -> bool {
        !self.connections.is_empty()
    }

    /// Get all connections of the participant
    pub fn connections(&self) -> impl Iterator<Item = ConnectionId> + use<'_> {
        self.connections.iter().map(|(conn, ..)| *conn)
    }

    /// Get a list of all connected devices
    pub fn devices(&self) -> BTreeSet<DeviceId> {
        let mut devices = BTreeSet::new();
        for device in self.connections.values() {
            devices.insert(*device);
        }
        devices
    }

    pub fn is_moderator(&self) -> bool {
        matches!(self.role, Role::Moderator)
    }
}
