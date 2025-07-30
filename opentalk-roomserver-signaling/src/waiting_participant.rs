// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::HashMap;

use opentalk_roomserver_types::{
    client_parameters::ClientParameters, connection_id::ConnectionId, device_id::DeviceId,
};

/// Information associated with a participant that joined the waiting room.
pub struct WaitingParticipant {
    pub connections: HashMap<ConnectionId, DeviceId>,
    pub client_parameters: ClientParameters,
    pub accepted: bool,
}
