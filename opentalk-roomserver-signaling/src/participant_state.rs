// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::{BTreeMap, BTreeSet, HashMap};

use chrono::{DateTime, Utc};
use opentalk_roomserver_types::{
    client_parameters::{ClientKind, Role},
    connection_id::ConnectionId,
    device_id::DeviceId,
    room_kind::RoomKind,
};
use opentalk_types_common::users::DisplayName;
use opentalk_types_signaling::{ParticipantId, ParticipationVisibility};
use serde::{Deserialize, Serialize};

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

    pub fn moderators(&self) -> ParticipantsFiltered {
        ParticipantsFiltered::new(self).moderators()
    }

    pub fn non_moderators(&self) -> ParticipantsFiltered {
        ParticipantsFiltered::new(self).non_moderators()
    }

    pub fn connections(&self) -> BTreeMap<ParticipantId, BTreeSet<ConnectionId>> {
        self.all_unfiltered
            .iter()
            .map(|(p, state)| (*p, state.connections.keys().copied().collect()))
            .collect()
    }

    pub fn visible(&self) -> ParticipantsFiltered {
        ParticipantsFiltered::new(self).visibility(ParticipationVisibility::Visible)
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

    /// The e-mail address of the participant
    pub email: Option<String>,

    /// The breakout room of the participant.
    pub room: RoomKind,

    /// The kind of the participant
    pub kind: ParticipantKind,

    /// The role that the participant assumes in the meeting.
    pub role: Role,

    /// All connections and their associated device
    pub connections: HashMap<ConnectionId, DeviceId>,

    /// The time the participant joined the meeting with their first connection
    pub joined_at: DateTime<Utc>,

    /// The time the participant left the meeting with their last connection
    pub left_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ParticipantKind {
    User,
    Guest,
    Recorder,
}

impl ParticipantKind {
    pub fn visibility(&self) -> ParticipationVisibility {
        match self {
            ParticipantKind::User | ParticipantKind::Guest => ParticipationVisibility::Visible,
            ParticipantKind::Recorder => ParticipationVisibility::Hidden,
        }
    }
}

impl From<&ClientKind> for ParticipantKind {
    fn from(value: &ClientKind) -> ParticipantKind {
        match value {
            ClientKind::Registered { .. } => ParticipantKind::User,
            ClientKind::Guest { .. } => ParticipantKind::Guest,
            ClientKind::Recorder => ParticipantKind::Recorder,
        }
    }
}

impl ParticipantState {
    pub fn new(
        display_name: DisplayName,
        email: Option<String>,
        kind: ParticipantKind,
        role: Role,
        joined_at: DateTime<Utc>,
    ) -> Self {
        Self {
            display_name,
            email,
            room: RoomKind::Main,
            kind,
            role,
            connections: HashMap::new(),
            joined_at,
            left_at: None,
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

    use chrono::Duration;
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
                email: None,
                room: RoomKind::Main,
                kind: ParticipantKind::User,
                role: Role::Moderator,
                connections: HashMap::from([(ConnectionId::generate(), DeviceId::nil())]),
                joined_at: DateTime::UNIX_EPOCH,
                left_at: None,
            },
        );

        let connected_participant_1 = ParticipantId::generate();
        participants.all_unfiltered.insert(
            connected_participant_1,
            ParticipantState {
                display_name: DisplayName::from_str_lossy("Connected 1"),
                email: None,
                room: RoomKind::Main,
                kind: ParticipantKind::Guest,
                role: Role::User,
                connections: HashMap::from([(ConnectionId::generate(), DeviceId::nil())]),
                joined_at: DateTime::UNIX_EPOCH,
                left_at: None,
            },
        );

        let disconnected_participant = ParticipantId::generate();
        participants.all_unfiltered.insert(
            disconnected_participant,
            ParticipantState {
                display_name: DisplayName::from_str_lossy("Disconnected"),
                email: None,
                room: RoomKind::Main,
                kind: ParticipantKind::User,
                role: Role::Moderator,
                connections: HashMap::from([]),
                joined_at: DateTime::UNIX_EPOCH,
                left_at: Some(DateTime::UNIX_EPOCH + Duration::hours(1)),
            },
        );

        assert_eq!(
            participants
                .connected()
                .ids()
                .collect::<HashSet<ParticipantId>>(),
            HashSet::from([connected_participant_0, connected_participant_1])
        );

        assert_eq!(
            participants
                .disconnected()
                .ids()
                .collect::<HashSet<ParticipantId>>(),
            HashSet::from([disconnected_participant])
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
                email: None,
                room: RoomKind::Breakout(BreakoutId::from(0)),
                kind: ParticipantKind::User,
                role: Role::Moderator,
                connections: HashMap::from([(ConnectionId::generate(), DeviceId::nil())]),
                joined_at: DateTime::UNIX_EPOCH,
                left_at: None,
            },
        );

        let disconnected_breakout = ParticipantId::generate();
        participants.all_unfiltered.insert(
            disconnected_breakout,
            ParticipantState {
                display_name: DisplayName::from_str_lossy("Connected 1"),
                email: None,
                room: RoomKind::Breakout(BreakoutId::from(0)),
                kind: ParticipantKind::Guest,
                role: Role::User,
                connections: HashMap::from([]),
                joined_at: DateTime::UNIX_EPOCH,
                left_at: Some(DateTime::UNIX_EPOCH + Duration::hours(1)),
            },
        );

        let disconnected_main = ParticipantId::generate();
        participants.all_unfiltered.insert(
            disconnected_main,
            ParticipantState {
                display_name: DisplayName::from_str_lossy("Disconnected"),
                email: None,
                room: RoomKind::Main,
                kind: ParticipantKind::User,
                role: Role::Moderator,
                connections: HashMap::from([]),
                joined_at: DateTime::UNIX_EPOCH,
                left_at: Some(DateTime::UNIX_EPOCH + Duration::hours(1)),
            },
        );

        // Participants in breakout room 0
        assert_eq!(
            participants
                .filter()
                .room(RoomKind::Breakout(BreakoutId::from(0)))
                .ids()
                .collect::<HashSet<ParticipantId>>(),
            HashSet::from([disconnected_breakout, connected_breakout])
        );

        // Connected participants in breakout room 0
        assert_eq!(
            participants
                .filter()
                .room(RoomKind::Breakout(BreakoutId::from(0)))
                .connected()
                .ids()
                .collect::<HashSet<ParticipantId>>(),
            HashSet::from([connected_breakout])
        );

        // Disconnected participants in breakout room 0
        assert_eq!(
            participants
                .filter()
                .room(RoomKind::Main)
                .disconnected()
                .ids()
                .collect::<HashSet<ParticipantId>>(),
            HashSet::from([disconnected_main])
        );

        // Participants in main room
        assert_eq!(
            participants
                .filter()
                .room(RoomKind::Main)
                .ids()
                .collect::<HashSet<ParticipantId>>(),
            HashSet::from([disconnected_main])
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
            HashSet::from([disconnected_main])
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
                email: None,
                room: RoomKind::Main,
                kind: ParticipantKind::User,
                role: Role::Moderator,
                connections: HashMap::from([(ConnectionId::generate(), DeviceId::nil())]),
                joined_at: DateTime::UNIX_EPOCH,
                left_at: None,
            },
        );

        let guest = ParticipantId::generate();
        participants.all_unfiltered.insert(
            guest,
            ParticipantState {
                display_name: DisplayName::from_str_lossy("Guest"),
                email: None,
                room: RoomKind::Main,
                kind: ParticipantKind::Guest,
                role: Role::User,
                connections: HashMap::new(),
                joined_at: DateTime::UNIX_EPOCH,
                left_at: Some(DateTime::UNIX_EPOCH + Duration::hours(1)),
            },
        );

        let recorder = ParticipantId::generate();
        participants.all_unfiltered.insert(
            recorder,
            ParticipantState {
                display_name: DisplayName::from_str_lossy("Recorder"),
                email: None,
                room: RoomKind::Main,
                kind: ParticipantKind::Recorder,
                role: Role::User,
                connections: HashMap::new(),
                joined_at: DateTime::UNIX_EPOCH,
                left_at: Some(DateTime::UNIX_EPOCH + Duration::hours(1)),
            },
        );

        assert_eq!(
            participants
                .filter()
                .visibility(ParticipationVisibility::Visible)
                .ids()
                .collect::<HashSet<ParticipantId>>(),
            HashSet::from([user, guest]),
        );

        assert_eq!(
            participants
                .filter()
                .visibility(ParticipationVisibility::Hidden)
                .ids()
                .collect::<HashSet<ParticipantId>>(),
            HashSet::from([recorder]),
        );
    }

    #[test]
    fn moderators() {
        let mut participants = Participants::new();

        let user = ParticipantId::generate();
        participants.all_unfiltered.insert(
            user,
            ParticipantState {
                display_name: DisplayName::from_str_lossy("User"),
                email: None,
                room: RoomKind::Main,
                kind: ParticipantKind::User,
                role: Role::Moderator,
                connections: HashMap::from([(ConnectionId::generate(), DeviceId::nil())]),
                joined_at: DateTime::UNIX_EPOCH,
                left_at: None,
            },
        );

        let guest = ParticipantId::generate();
        participants.all_unfiltered.insert(
            guest,
            ParticipantState {
                display_name: DisplayName::from_str_lossy("Guest"),
                email: None,
                room: RoomKind::Main,
                kind: ParticipantKind::Guest,
                role: Role::User,
                connections: HashMap::new(),
                joined_at: DateTime::UNIX_EPOCH,
                left_at: Some(DateTime::UNIX_EPOCH + Duration::hours(1)),
            },
        );

        let recorder = ParticipantId::generate();
        participants.all_unfiltered.insert(
            recorder,
            ParticipantState {
                display_name: DisplayName::from_str_lossy("Recorder"),
                email: None,
                room: RoomKind::Main,
                kind: ParticipantKind::Recorder,
                role: Role::User,
                connections: HashMap::new(),
                joined_at: DateTime::UNIX_EPOCH,
                left_at: Some(DateTime::UNIX_EPOCH + Duration::hours(1)),
            },
        );

        assert_eq!(
            participants
                .filter()
                .moderators()
                .ids()
                .collect::<HashSet<ParticipantId>>(),
            HashSet::from([user]),
        );
    }

    #[test]
    fn non_moderators() {
        let mut participants = Participants::new();

        let user = ParticipantId::generate();
        participants.all_unfiltered.insert(
            user,
            ParticipantState {
                display_name: DisplayName::from_str_lossy("User"),
                email: None,
                room: RoomKind::Main,
                kind: ParticipantKind::User,
                role: Role::Moderator,
                connections: HashMap::from([(ConnectionId::generate(), DeviceId::nil())]),
                joined_at: DateTime::UNIX_EPOCH,
                left_at: None,
            },
        );

        let guest = ParticipantId::generate();
        participants.all_unfiltered.insert(
            guest,
            ParticipantState {
                display_name: DisplayName::from_str_lossy("Guest"),
                email: None,
                room: RoomKind::Main,
                kind: ParticipantKind::Guest,
                role: Role::User,
                connections: HashMap::new(),
                joined_at: DateTime::UNIX_EPOCH,
                left_at: None,
            },
        );

        let recorder = ParticipantId::generate();
        participants.all_unfiltered.insert(
            recorder,
            ParticipantState {
                display_name: DisplayName::from_str_lossy("Recorder"),
                email: None,
                room: RoomKind::Main,
                kind: ParticipantKind::Recorder,
                role: Role::User,
                connections: HashMap::new(),
                joined_at: DateTime::UNIX_EPOCH,
                left_at: Some(DateTime::UNIX_EPOCH + Duration::hours(1)),
            },
        );

        assert_eq!(
            participants
                .filter()
                .non_moderators()
                .ids()
                .collect::<HashSet<ParticipantId>>(),
            HashSet::from([guest, recorder]),
        );
    }
}
