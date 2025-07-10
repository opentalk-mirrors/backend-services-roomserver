// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{collections::HashMap, sync::Arc};

use opentalk_roomserver_common::{application_state::ApplicationState, settings::Settings};
use opentalk_roomserver_signaling::storage::StorageProvider;
use opentalk_roomserver_types::room_parameters::RoomParameters;
use opentalk_roomserver_web_api::v1::{RoomAction, signaling::websocket::SignalingSocket};
use opentalk_types_common::rooms::RoomId;
use tokio::{
    sync::{RwLock, watch},
    task::JoinHandle,
};

use super::signaling::module_initializer::ModuleRegistry;
use crate::task::{
    RoomTask,
    fs_storage::FsStorage,
    handle::{RoomTaskHandle, RoomTaskHandleError},
};

/// The room task registry
///
/// Holds a list over all active rooms and their [`RoomTaskHandle`].
#[derive(Default, Debug)]
pub struct RoomTaskRegistry<Socket: SignalingSocket + 'static> {
    inner: Arc<RwLock<HashMap<RoomId, RoomTaskHandle<Socket>>>>,
}

// Manually implementing clone so that we don't require [`Socket`] to be
// Clone as well.
impl<Socket: SignalingSocket> Clone for RoomTaskRegistry<Socket> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Socket: SignalingSocket> RoomTaskRegistry<Socket> {
    /// Creates a new [`RoomTaskRegistry`] wi th default values
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    /// Spawns a new room task and adds it to the registry.
    ///
    /// Returns [`Created`] when a new room was created otherwise [`Updated`] is returned.
    ///
    /// [`Created`]: RoomAction::Created
    /// [`Updated`]: RoomAction::Updated
    pub async fn put_room(
        &self,
        room_id: RoomId,
        room_parameters: Arc<RoomParameters>,
        module_registry: Arc<ModuleRegistry>,
        settings: Arc<Settings>,
        app_state: watch::Receiver<ApplicationState>,
    ) -> Result<(RoomAction, RoomTaskHandle<Socket>), RoomTaskHandleError<Socket>> {
        let registry = self.inner.write().await;

        if let Some(task_handle) = registry.get(&room_id) {
            task_handle
                .update_parameter((*room_parameters).clone())
                .await?;
            return Ok((RoomAction::Updated, task_handle.clone()));
        }

        let storage = create_storage_provider();

        let (task_handle, join_handle) = RoomTask::spawn(
            room_id,
            room_parameters,
            module_registry,
            storage,
            settings,
            app_state,
        );

        self.insert(room_id, registry, &task_handle, join_handle);

        Ok((RoomAction::Created, task_handle))
    }

    fn insert(
        &self,
        room_id: RoomId,
        mut registry: tokio::sync::RwLockWriteGuard<'_, HashMap<RoomId, RoomTaskHandle<Socket>>>,
        task_handle: &RoomTaskHandle<Socket>,
        join_handle: JoinHandle<()>,
    ) {
        registry.insert(room_id, task_handle.clone());
        let this = self.clone();
        tokio::spawn(async move {
            let _ = join_handle.await;
            this.remove_room(room_id).await;
        });
    }

    /// Spawns a new room task or returns the [`RoomTaskHandle`] if the room task is already running.
    ///
    /// Returns [`None`] when the room was created.
    pub async fn create_or_get(
        &self,
        room_id: RoomId,
        room_parameters: Arc<RoomParameters>,
        module_registry: Arc<ModuleRegistry>,
        settings: Arc<Settings>,
        app_state: watch::Receiver<ApplicationState>,
    ) -> Option<RoomTaskHandle<Socket>> {
        let registry = self.inner.write().await;

        if let Some(task_handle) = registry.get(&room_id) {
            return Some(task_handle.clone());
        }

        let storage = create_storage_provider();

        let (task_handle, join_handle) = RoomTask::spawn(
            room_id,
            room_parameters,
            module_registry,
            storage,
            settings,
            app_state,
        );
        self.insert(room_id, registry, &task_handle, join_handle);

        None
    }

    /// Checks if the requested room id exists and refreshes the idle timeout if it does
    pub async fn ensure_room_exists(&self, room_id: &RoomId) -> bool {
        let registry = self.inner.read().await;

        let Some(handle) = registry.get(room_id) else {
            return false;
        };

        match handle.refresh_idle_timeout().await {
            Ok(_) => true,
            Err(RoomTaskHandleError::Gone { .. }) => false,
            Err(e) => {
                log::error!("Unexpected error while refreshing idle timeout: {e}");
                false
            }
        }
    }

    /// Get the [`RoomTaskHandle`] for the specified [`RoomId`]
    pub async fn get_task_handle(&self, room_id: &RoomId) -> Option<RoomTaskHandle<Socket>> {
        self.inner.read().await.get(room_id).cloned()
    }

    /// Removes the room from the registry
    ///
    /// This will also destroy the related [`RoomTask`]
    pub async fn remove_room(&self, room_id: RoomId) {
        let mut room_list = self.inner.write().await;

        log::trace!("Remove room task handle from registry: {room_id}");
        room_list.remove(&room_id);
    }
}

// TODO: this function will be replaced once a real storage provider has been implemented
fn create_storage_provider() -> Arc<dyn StorageProvider> {
    let quota = 5 * 1024u64.pow(3); // 5 GiB
    let storage = FsStorage::new(quota, None).expect("Failed to create storage");
    Arc::new(storage)
}
