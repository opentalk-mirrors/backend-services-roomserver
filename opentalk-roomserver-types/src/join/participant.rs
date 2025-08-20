// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::{ModulePeerData, ParticipantId, SignalingModulePeerFrontendData};
use serde::{Deserialize, Serialize};

use crate::join::connection_info::ConnectionInfo;

/// Status information about a participant
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Participant {
    /// The id of the participant
    pub id: ParticipantId,

    /// The connections of the participant
    pub connections: Vec<ConnectionInfo>,

    /// Module data for the participant
    pub module_data: ModulePeerData,
}

impl Participant {
    /// Gets the inner module data of a Participant
    pub fn get_module<T: SignalingModulePeerFrontendData>(
        &self,
    ) -> Result<Option<T>, serde_json::Error> {
        self.module_data.get::<T>()
    }

    /// Updates the inner module data of a Participant and returns the new data
    pub fn update_module<T: SignalingModulePeerFrontendData, F: FnOnce(&mut T)>(
        &mut self,
        update: F,
    ) -> Result<Option<T>, serde_json::Error> {
        self.module_data.update::<T, F>(update)
    }
}

#[cfg(test)]
mod test {
    use opentalk_types_signaling::{ModulePeerData, ParticipantId};

    use crate::{
        breakout::module_data::BreakoutPeerModuleData,
        connection_id::ConnectionId,
        device_id::DeviceId,
        join::{connection_info::ConnectionInfo, participant::Participant},
    };

    #[test]
    fn serialize() {
        let mut participant = Participant {
            id: ParticipantId::from_u128(0x0001),
            connections: vec![ConnectionInfo {
                connection_id: ConnectionId::from_u128(0x0101),
                device_id: DeviceId::from_u128(0x0201),
            }],
            module_data: ModulePeerData::new(),
        };

        participant
            .module_data
            .insert(&BreakoutPeerModuleData {
                room: crate::room_kind::RoomKind::Main,
            })
            .unwrap();

        insta::assert_json_snapshot!(participant, @r#"
        {
          "id": "00000000-0000-0000-0000-000000000001",
          "connections": [
            {
              "connection_id": "00000000-0000-0000-0000-000000000101",
              "device_id": "00000000-0000-0000-0000-000000000201"
            }
          ],
          "breakout": {
            "room": {
              "kind": "main"
            }
          }
        }
        "#);
    }
}
