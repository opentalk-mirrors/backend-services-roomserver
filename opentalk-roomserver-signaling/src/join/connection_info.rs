// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types::{connection_id::ConnectionId, device_id::DeviceId};
use serde::{Deserialize, Serialize};

/// A participants connection information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub connection_id: ConnectionId,
    pub device_id: DeviceId,
}
