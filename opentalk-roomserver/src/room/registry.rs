// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{collections::HashMap, sync::Arc};

use opentalk_roomserver_types::room_parameters::RoomParameters;
use opentalk_roomserver_web_api::v1::RoomAction;
use opentalk_types_common::rooms::RoomId;
use tokio::sync::RwLock;

use super::{handle::RoomTaskHandle, task::RoomTask};

/// The room task registry
///
/// Holds a list over all active rooms and their [`RoomTaskHandle`].
#[derive(Clone, Default, Debug)]
pub(crate) struct RoomTaskRegistry {
    inner: Arc<RwLock<HashMap<RoomId, RoomTaskHandle>>>,
}

impl RoomTaskRegistry {
    /// Spawns a new room task and adds it to the registry.
    ///
    /// Returns [`Created`] when a new room was created otherwise [`Updated`] is returned.
    ///
    /// [`Created`]: RoomAction::Created
    /// [`Updated`]: RoomAction::Updated
    pub(crate) async fn put_room(
        &self,
        room_id: RoomId,
        room_parameters: RoomParameters,
    ) -> anyhow::Result<(RoomAction, RoomTaskHandle)> {
        let mut registry = self.inner.write().await;

        if let Some(task_handle) = registry.get(&room_id) {
            task_handle.update_parameter(room_parameters).await?;
            return Ok((RoomAction::Updated, task_handle.clone()));
        }

        let task_handle = RoomTask::spawn(room_id, room_parameters, self.clone());

        registry.insert(room_id, task_handle.clone());

        Ok((RoomAction::Created, task_handle))
    }

    /// Get the [`RoomTaskHandle`] for the specified [`RoomId`]
    #[allow(dead_code)] //TODO: remove when used
    pub(crate) async fn get_task_handle(&self, room_id: &RoomId) -> Option<RoomTaskHandle> {
        self.inner.read().await.get(room_id).cloned()
    }

    /// Removes the room from the registry
    ///
    /// This will also destroy the related [`RoomTask`]
    pub(crate) async fn remove_room(&self, room_id: RoomId) {
        let mut room_list = self.inner.write().await;

        log::trace!("Remove room task handle from registry: {room_id}");
        room_list.remove(&room_id);
    }
}
