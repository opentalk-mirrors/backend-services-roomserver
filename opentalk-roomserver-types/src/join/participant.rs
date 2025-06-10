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
    #[serde(flatten)]
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
