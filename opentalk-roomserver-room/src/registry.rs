// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::{HashMap, HashSet},
    pin::Pin,
    sync::Arc,
    time::Duration,
};

use opentalk_orchestrator_client::{RoomServerEvent, client::OrchestratorHandle};
use opentalk_roomserver_common::{application_state::ApplicationState, settings::Settings};
use opentalk_roomserver_types::room_parameters::RoomParameters;
use opentalk_roomserver_web_api::v1::{RoomAction, signaling::websocket::SignalingSocket};
use opentalk_types_common::rooms::RoomId;
use tokio::sync::{Notify, RwLock, watch};

use super::signaling::module_initializer::ModuleRegistry;
use crate::task::{
    RoomTask,
    handle::{RoomTaskHandle, RoomTaskHandleError},
};

/// The room task registry
///
/// Holds a list over all active rooms and their [`RoomTaskHandle`].
#[derive(Default, Debug)]
pub struct RoomTaskRegistry<Socket: SignalingSocket + 'static> {
    inner: Arc<RwLock<HashMap<RoomId, RoomTaskHandle<Socket>>>>,
    room_removed: Arc<Notify>,
    idle_timeout: Duration,
    orchestrator_handle: Option<OrchestratorHandle>,
}

// Manually implementing clone so that we don't require [`Socket`] to be
// Clone as well.
impl<Socket: SignalingSocket> Clone for RoomTaskRegistry<Socket> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            room_removed: self.room_removed.clone(),
            idle_timeout: self.idle_timeout,
            orchestrator_handle: self.orchestrator_handle.clone(),
        }
    }
}

impl<Socket: SignalingSocket> RoomTaskRegistry<Socket> {
    /// Creates a new [`RoomTaskRegistry`] wi th default values
    pub fn new(idle_timeout: Duration, orchestrator_handle: Option<OrchestratorHandle>) -> Self {
        Self {
            inner: Arc::default(),
            room_removed: Arc::default(),
            idle_timeout,
            orchestrator_handle,
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
                .set_parameters((*room_parameters).clone())
                .await?;
            return Ok((RoomAction::Updated, task_handle.clone()));
        }

        let (task_handle, future_room) = RoomTask::setup(
            room_id,
            room_parameters,
            module_registry,
            settings,
            app_state,
            self.idle_timeout,
        );

        self.insert(room_id, registry, &task_handle, future_room)
            .await;

        Ok((RoomAction::Created, task_handle))
    }

    async fn insert(
        &self,
        room_id: RoomId,
        mut registry: tokio::sync::RwLockWriteGuard<'_, HashMap<RoomId, RoomTaskHandle<Socket>>>,
        task_handle: &RoomTaskHandle<Socket>,
        future_room: Pin<Box<dyn Future<Output = ()> + Send>>,
    ) {
        let this = self.clone();

        tokio::spawn(async move {
            future_room.await;
            this.remove_room(room_id).await;
        });

        registry.insert(room_id, task_handle.clone());
    }

    /// Spawns a new room task if it does not already exists
    pub async fn create_if_not_exists(
        &self,
        room_id: RoomId,
        room_parameters: Arc<RoomParameters>,
        module_registry: Arc<ModuleRegistry>,
        settings: Arc<Settings>,
        app_state: watch::Receiver<ApplicationState>,
    ) {
        let registry = self.inner.write().await;

        if registry.get(&room_id).is_some() {
            // Room already exists
            return;
        }

        let (task_handle, join_handle) = RoomTask::setup(
            room_id,
            room_parameters,
            module_registry,
            settings,
            app_state,
            self.idle_timeout,
        );

        self.insert(room_id, registry, &task_handle, join_handle)
            .await;
    }

    /// Checks if the requested room id exists and refreshes the idle timeout if it does
    pub async fn ensure_room_exists(&self, room_id: &RoomId) -> bool {
        let registry = self.inner.read().await;

        let Some(handle) = registry.get(room_id) else {
            return false;
        };

        match handle.refresh_idle_timeout().await {
            Ok(()) => true,
            Err(RoomTaskHandleError::Gone { .. }) => false,
            Err(e) => {
                tracing::error!("Unexpected error while refreshing idle timeout: {e}");
                false
            }
        }
    }

    pub async fn allowed_origins(&self, room_id: RoomId) -> Option<Vec<String>> {
        let registry = self.inner.read().await;
        registry.get(&room_id)?.allowed_origins().await
    }

    /// Get the [`RoomTaskHandle`] for the specified [`RoomId`]
    pub async fn get_task_handle(&self, room_id: &RoomId) -> Option<RoomTaskHandle<Socket>> {
        self.inner.read().await.get(room_id).cloned()
    }

    /// Removes the room from the registry
    ///
    /// This will also destroy the related [`RoomTask`]
    async fn remove_room(&self, room_id: RoomId) {
        tracing::trace!("Remove room task handle from registry: {room_id}");
        let mut room_list = self.inner.write().await;

        room_list.remove(&room_id);
        self.room_removed.notify_waiters();

        if let Some(handle) = &self.orchestrator_handle
            && let Err(e) = handle
                .send_event(RoomServerEvent::RemoveRoom(room_id))
                .await
        {
            tracing::error!("Failed to notify orchestrator about removed room: {e}");
        }
    }

    /// Returns all known room ids
    pub async fn room_ids(&self) -> HashSet<RoomId> {
        self.inner.read().await.keys().copied().collect()
    }

    /// Wait until all pending room task have finished.
    ///
    /// ## Warning
    ///
    /// Use this function with care, as this will acquire a lock for `pending_room_tasks`.
    /// This will block new tasks from being added to the pending tasks.
    pub async fn wait_for_room_closed(&self) {
        // Wait for all room tasks to finish, acquire the lock in every iteration to not cause a
        // deadlock
        while !self.inner.read().await.is_empty() {
            tracing::trace!(
                "Wait for room task removal, remaining tasks: {}",
                self.inner.read().await.len()
            );
            self.room_removed.notified().await;
        }
    }
}

#[cfg(test)]
#[cfg(feature = "mock")]
mod tests {
    use std::time::Duration;

    use opentalk_roomserver_common::{application_state::ApplicationState, settings::Settings};
    use opentalk_roomserver_types::room_parameters::RoomParameters;
    use opentalk_roomserver_web_api::v1::RoomAction;
    use opentalk_types_common::{rooms::RoomId, utils::ExampleData as _};

    use crate::{ModuleRegistry, RoomTaskRegistry, mocking::socket::MockSocket};

    #[test_log::test(tokio::test)]
    async fn room_is_removed_after_idle_timeout() {
        let registry = RoomTaskRegistry::<MockSocket>::new(Duration::from_secs(0), None);
        let (_app_state_sender, app_state) = tokio::sync::watch::channel(ApplicationState::Running);

        let room_id = RoomId::from_u128(1);
        // build room parameter without any modules
        let mut parameter = RoomParameters::example_data();
        parameter.module_settings.retain(|_, _| false);

        let (action, ..) = registry
            .put_room(
                room_id,
                parameter.into(),
                ModuleRegistry::new().into(),
                Settings::test_settings("secret".to_string()).into(),
                app_state,
            )
            .await
            .unwrap();

        assert_eq!(RoomAction::Created, action);

        tokio::time::timeout(Duration::from_millis(500), registry.wait_for_room_closed())
            .await
            .unwrap();

        let handle = registry.get_task_handle(&room_id).await;
        assert!(handle.is_none());
    }

    #[test_log::test(tokio::test)]
    async fn room_is_removed_after_shutdown() {
        // use a high RoomTask idle timeout to prevent stopping because of the timeout.
        let registry = RoomTaskRegistry::<MockSocket>::new(Duration::from_secs(99999), None);
        let (app_state_sender, app_state) = tokio::sync::watch::channel(ApplicationState::Running);

        let room_id = RoomId::from_u128(1);
        // build room parameter without any modules
        let mut parameter = RoomParameters::example_data();
        parameter.module_settings.retain(|_, _| false);

        let (action, ..) = registry
            .put_room(
                room_id,
                parameter.into(),
                ModuleRegistry::new().into(),
                Settings::test_settings("secret".to_string()).into(),
                app_state,
            )
            .await
            .unwrap();

        assert_eq!(RoomAction::Created, action);
        app_state_sender
            .send(ApplicationState::ShuttingDown)
            .unwrap();

        tokio::time::timeout(Duration::from_millis(500), registry.wait_for_room_closed())
            .await
            .unwrap();

        let handle = registry.get_task_handle(&room_id).await;
        assert!(handle.is_none());
    }
}
