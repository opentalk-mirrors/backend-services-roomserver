// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::HashMap;

use opentalk_roomserver_types::{
    client_parameters::{ClientKind, Role},
    connection_id::ConnectionId,
    device_id::DeviceId,
};

/// Information associated with a participant that joined the waiting room.
#[derive(Debug, Clone)]
pub struct WaitingParticipant {
    /// The kind of the participant
    pub kind: ClientKind,

    /// The role that the participant assumes in the meeting.
    pub role: Role,

    /// All connections and their associated devices
    pub connections: HashMap<ConnectionId, DeviceId>,

    pub accepted: bool,
}
