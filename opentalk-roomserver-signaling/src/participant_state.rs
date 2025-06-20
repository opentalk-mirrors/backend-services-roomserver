// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::{BTreeMap, BTreeSet, HashMap};

use opentalk_roomserver_types::{
    client_parameters::{Role, ServiceKind},
    connection_id::ConnectionId,
    device_id::DeviceId,
    room_kind::RoomKind,
};
use opentalk_types_common::users::DisplayName;
use opentalk_types_signaling::{ParticipantId, ParticipationVisibility};

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
    Service(ServiceKind),
}

impl ParticipantKind {
    pub fn visibility(&self) -> ParticipationVisibility {
        match self {
            ParticipantKind::User | ParticipantKind::Guest => ParticipationVisibility::Visible,
            ParticipantKind::Service(service_kind) => service_kind.visibility(),
        }
    }
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

    pub fn is_visible(&self) -> bool {
        self.kind.visibility().is_visible()
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use opentalk_roomserver_types::breakout::breakout_id::BreakoutId;

    use super::*;

    #[test]
    fn filter_connected() {
        let mut participants = Participants::new();

        let connected_participant_0 = ParticipantId::generate();
        participants.all_unfiltered.insert(
            connected_participant_0,
            ParticipantState {
                display_name: DisplayName::from_str_lossy("Connected 0"),
                room: RoomKind::Main,
                kind: ParticipantKind::User,
                role: Role::Moderator,
                connections: HashMap::from_iter([(ConnectionId::generate(), DeviceId::nil())]),
            },
        );

        let connected_participant_1 = ParticipantId::generate();
        participants.all_unfiltered.insert(
            connected_participant_1,
            ParticipantState {
                display_name: DisplayName::from_str_lossy("Connected 1"),
                room: RoomKind::Main,
                kind: ParticipantKind::Guest,
                role: Role::User,
                connections: HashMap::from_iter([(ConnectionId::generate(), DeviceId::nil())]),
            },
        );

        let disconnected_participant = ParticipantId::generate();
        participants.all_unfiltered.insert(
            disconnected_participant,
            ParticipantState {
                display_name: DisplayName::from_str_lossy("Disconnected"),
                room: RoomKind::Main,
                kind: ParticipantKind::User,
                role: Role::Moderator,
                connections: HashMap::from_iter([]),
            },
        );

        assert_eq!(
            participants
                .connected()
                .ids()
                .collect::<HashSet<ParticipantId>>(),
            HashSet::from_iter([connected_participant_0, connected_participant_1])
        );

        assert_eq!(
            participants
                .disconnected()
                .ids()
                .collect::<HashSet<ParticipantId>>(),
            HashSet::from_iter([disconnected_participant])
        );
    }

    #[test]
    fn filter_room() {
        let mut participants = Participants::new();

        let connected_breakout = ParticipantId::generate();
        participants.all_unfiltered.insert(
            connected_breakout,
            ParticipantState {
                display_name: DisplayName::from_str_lossy("Connected 0"),
                room: RoomKind::Breakout(BreakoutId::from(0)),
                kind: ParticipantKind::User,
                role: Role::Moderator,
                connections: HashMap::from_iter([(ConnectionId::generate(), DeviceId::nil())]),
            },
        );

        let disconnected_breakout = ParticipantId::generate();
        participants.all_unfiltered.insert(
            disconnected_breakout,
            ParticipantState {
                display_name: DisplayName::from_str_lossy("Connected 1"),
                room: RoomKind::Breakout(BreakoutId::from(0)),
                kind: ParticipantKind::Guest,
                role: Role::User,
                connections: HashMap::from_iter([]),
            },
        );

        let disconnected_main = ParticipantId::generate();
        participants.all_unfiltered.insert(
            disconnected_main,
            ParticipantState {
                display_name: DisplayName::from_str_lossy("Disconnected"),
                room: RoomKind::Main,
                kind: ParticipantKind::User,
                role: Role::Moderator,
                connections: HashMap::from_iter([]),
            },
        );

        // Participants in breakout room 0
        assert_eq!(
            participants
                .filter()
                .room(RoomKind::Breakout(BreakoutId::from(0)))
                .ids()
                .collect::<HashSet<ParticipantId>>(),
            HashSet::from_iter([disconnected_breakout, connected_breakout])
        );

        // Connected participants in breakout room 0
        assert_eq!(
            participants
                .filter()
                .room(RoomKind::Breakout(BreakoutId::from(0)))
                .connected()
                .ids()
                .collect::<HashSet<ParticipantId>>(),
            HashSet::from_iter([connected_breakout])
        );

        // Disconnected participants in breakout room 0
        assert_eq!(
            participants
                .filter()
                .room(RoomKind::Main)
                .disconnected()
                .ids()
                .collect::<HashSet<ParticipantId>>(),
            HashSet::from_iter([disconnected_main])
        );

        // Participants in main room
        assert_eq!(
            participants
                .filter()
                .room(RoomKind::Main)
                .ids()
                .collect::<HashSet<ParticipantId>>(),
            HashSet::from_iter([disconnected_main])
        );

        // Connected participants in main room
        assert!(
            participants
                .filter()
                .room(RoomKind::Main)
                .connected()
                .iter()
                .next()
                .is_none()
        );

        // Disconnected participants in main room
        assert_eq!(
            participants
                .filter()
                .room(RoomKind::Main)
                .disconnected()
                .ids()
                .collect::<HashSet<ParticipantId>>(),
            HashSet::from_iter([disconnected_main])
        );
    }

    #[test]
    fn filter_visibility() {
        let mut participants = Participants::new();

        let user = ParticipantId::generate();
        participants.all_unfiltered.insert(
            user,
            ParticipantState {
                display_name: DisplayName::from_str_lossy("User"),
                room: RoomKind::Main,
                kind: ParticipantKind::User,
                role: Role::Moderator,
                connections: HashMap::from_iter([(ConnectionId::generate(), DeviceId::nil())]),
            },
        );

        let guest = ParticipantId::generate();
        participants.all_unfiltered.insert(
            guest,
            ParticipantState {
                display_name: DisplayName::from_str_lossy("Guest"),
                room: RoomKind::Main,
                kind: ParticipantKind::Guest,
                role: Role::User,
                connections: HashMap::new(),
            },
        );

        let recorder = ParticipantId::generate();
        participants.all_unfiltered.insert(
            recorder,
            ParticipantState {
                display_name: DisplayName::from_str_lossy("Recorder"),
                room: RoomKind::Main,
                kind: ParticipantKind::Service(ServiceKind::Recorder),
                role: Role::User,
                connections: HashMap::new(),
            },
        );

        assert_eq!(
            participants
                .filter()
                .visibility(ParticipationVisibility::Visible)
                .ids()
                .collect::<HashSet<ParticipantId>>(),
            HashSet::from_iter([user, guest]),
        );

        assert_eq!(
            participants
                .filter()
                .visibility(ParticipationVisibility::Hidden)
                .ids()
                .collect::<HashSet<ParticipantId>>(),
            HashSet::from_iter([recorder]),
        );
    }
}
