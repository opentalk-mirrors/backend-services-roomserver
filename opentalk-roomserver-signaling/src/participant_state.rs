// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::{BTreeSet, HashMap};

use opentalk_roomserver_types::{
    breakout_id::BreakoutId, client_parameters::Role, connection_id::ConnectionId,
    device_id::DeviceId,
};
use opentalk_types_common::users::DisplayName;
use opentalk_types_signaling::ParticipantId;

#[derive(Debug, Default)]
pub struct Participants {
    pub breakout_scope: Option<Option<BreakoutId>>,

    /// Contains all connected and disconnected participants, even those outside of the current breakout scope
    pub all_unfiltered: HashMap<ParticipantId, ParticipantState>,
}

impl Participants {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn all(&self) -> impl Iterator<Item = (&ParticipantId, &ParticipantState)> {
        self.all_unfiltered
            .iter()
            .filter(|(_, s)| self.breakout_filter(s))
    }

    pub fn connected(&self) -> impl Iterator<Item = (&ParticipantId, &ParticipantState)> {
        self.all_unfiltered
            .iter()
            .filter(|(_, s)| self.breakout_filter(s) && s.is_connected())
    }

    pub fn disconnected(&self) -> impl Iterator<Item = (&ParticipantId, &ParticipantState)> {
        self.all_unfiltered
            .iter()
            .filter(|(_, s)| self.breakout_filter(s) && !s.is_connected())
    }

    pub fn get(&self, participant_id: &ParticipantId) -> Option<&ParticipantState> {
        self.all_unfiltered
            .get(participant_id)
            .filter(|s| self.breakout_filter(s))
    }

    pub fn get_connected(&self, participant_id: &ParticipantId) -> Option<&ParticipantState> {
        self.all_unfiltered
            .get(participant_id)
            .filter(|s| self.breakout_filter(s) && s.is_connected())
    }

    pub fn get_disconnected(&self, participant_id: &ParticipantId) -> Option<&ParticipantState> {
        self.all_unfiltered
            .get(participant_id)
            .filter(|s| self.breakout_filter(s) && !s.is_connected())
    }

    fn breakout_filter(&self, state: &ParticipantState) -> bool {
        match self.breakout_scope {
            Some(scope) => scope == state.breakout_room,
            None => true,
        }
    }
}

#[derive(Debug)]
pub struct ParticipantState {
    /// The participants display name
    pub display_name: DisplayName,

    /// The breakout room of the participant. Is `None` when in the main room
    pub breakout_room: Option<BreakoutId>,

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
            breakout_room: None,
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
