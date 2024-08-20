// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{collections::HashMap, sync::Arc};

use opentalk_types::core::RoomId;
use parking_lot::RwLock;

use opentalk_web_api::types::RoomParameters;

use super::{handle::RoomTaskHandle, task::RoomTask};

/// The room task registry
///
/// Holds a list over all active rooms and their [`RoomTaskHandle`].
#[derive(Clone, Default, Debug)]
pub(crate) struct RoomTaskRegistry {
    inner: Arc<RwLock<HashMap<RoomId, RoomTaskHandle>>>,
}

impl RoomTaskRegistry {
    /// Spawns a new room task and adds it to the registry
    ///
    /// Returns `true` when a new room was created
    pub(crate) fn create_room_if_not_exists(
        &self,
        room_parameters: RoomParameters,
    ) -> (bool, RoomTaskHandle) {
        let room_id = room_parameters.room_id;

        let mut registry = self.inner.write();

        if let Some(task_handle) = registry.get(&room_id) {
            return (false, task_handle.clone());
        }

        let task_handle = RoomTask::spawn(room_parameters, self.clone());

        registry.insert(room_id, task_handle.clone());

        (true, task_handle)
    }

    /// Get the [`RoomTaskHandle`] for the specified [`RoomId`]
    #[allow(dead_code)] //TODO: remove when used
    pub(crate) fn get_task_handle(&self, room_id: &RoomId) -> Option<RoomTaskHandle> {
        self.inner.read().get(room_id).cloned()
    }

    /// Removes the room from the registry
    ///
    /// This will also destroy the related [`RoomTask`]
    pub(crate) fn remove_room(&self, room_id: RoomId) {
        let mut room_list = self.inner.write();

        room_list.remove(&room_id);
    }
}
