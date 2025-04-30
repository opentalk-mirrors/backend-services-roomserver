// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::{BTreeSet, HashMap};

use opentalk_roomserver_types::{
    client_parameters::Role, connection_id::ConnectionId, device_id::DeviceId,
};
use opentalk_types_common::users::DisplayName;
use opentalk_types_signaling::ParticipantId;

#[derive(Debug, Default)]
pub struct Participants {
    /// Contains all connected and disconnected participants
    pub all: HashMap<ParticipantId, ParticipantState>,
}

impl Participants {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn connected(&self) -> impl Iterator<Item = (&ParticipantId, &ParticipantState)> {
        self.all.iter().filter(|(_, s)| s.is_connected())
    }

    pub fn get_connected(&self, participant_id: &ParticipantId) -> Option<&ParticipantState> {
        self.all.get(participant_id).filter(|s| s.is_connected())
    }

    pub fn disconnected(&self) -> impl Iterator<Item = (&ParticipantId, &ParticipantState)> {
        self.all.iter().filter(|(_, s)| !s.is_connected())
    }

    pub fn get_disconnected(&self, participant_id: &ParticipantId) -> Option<&ParticipantState> {
        self.all.get(participant_id).filter(|s| !s.is_connected())
    }
}

#[derive(Debug)]
pub struct ParticipantState {
    /// The participants display name
    pub display_name: DisplayName,

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
}
