// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! Contains code for room management and signaling.
//!
//! The room state is managed by the [`task::RoomTask`], where each room has its own [`tokio::task`] with an instance of
//! a [`RoomTask`](task::RoomTask). The [`RoomTasks`](task::RoomTask) have a channel interface that is exposed via the
//! [`RoomTaskHandle`] through which the web api can send requests to each
//! individual room.
//!
//! The active rooms are created and tracked with the [`RoomTaskRegistry`]. When a
//! [`task::RoomTask`] gets destroyed, it removes itself from the [`RoomTaskRegistry`].

pub mod message_router;
#[cfg(any(test, feature = "mock"))]
pub mod mocking;
pub mod registry;
pub mod signaling;
pub mod task;

pub use opentalk_roomserver_signaling::storage::{
    AssetMetaData, AssetUploaded, StorageError, StorageProvider,
};

pub use crate::{
    registry::RoomTaskRegistry,
    signaling::module_initializer::ModuleRegistry,
    task::{
        RoomTaskApiError,
        handle::{Request, RoomTaskHandle, RoomTaskHandleError},
    },
};

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use opentalk_roomserver_common::{application_state::ApplicationState, settings::Settings};
    use opentalk_roomserver_types::{
        client_parameters::{self, ClientParameters},
        room_parameters::RoomParameters,
    };
    use opentalk_types_common::{
        rooms::RoomId, roomserver::DeviceSecret, users::DisplayName, utils::ExampleData,
    };
    use tokio::{sync::watch, time::sleep};

    use super::{signaling::module_initializer::ModuleRegistry, task::handle::RoomTaskHandle};
    use crate::{
        mocking::{participant::create_participant_connection, socket::MockSocket},
        task::{RoomTask, fs_storage::FsStorage},
    };

    const TIMEOUT: Duration = Duration::from_millis(500);

    fn create_room_task() -> (RoomTaskHandle<MockSocket>, watch::Sender<ApplicationState>) {
        let id = RoomId::from_u128(0xc270ab35_5cdb_4614_b872_8dd66ceefc70);
        let params = Arc::new(RoomParameters::example_data());
        let module_registry = Arc::new(ModuleRegistry::new());
        let (sender, state) = watch::channel(ApplicationState::Running);
        let settings = Arc::new(Settings::test_settings("secret".to_owned()));
        let storage = FsStorage::new(1024, None).expect("Failed to create storage");
        let storage = Arc::new(storage);
        (
            RoomTask::spawn_with_timeout(
                id,
                params,
                state,
                module_registry,
                storage,
                settings,
                TIMEOUT,
            )
            .0,
            sender,
        )
    }

    #[tokio::test]
    async fn timeout() {
        let (handle, _sender) = create_room_task();
        sleep(TIMEOUT - Duration::from_millis(100)).await;
        handle.refresh_idle_timeout().await.unwrap();
        sleep(TIMEOUT + Duration::from_millis(100)).await;
        handle.refresh_idle_timeout().await.unwrap_err();
    }

    #[tokio::test]
    async fn accept_signaling_socket() {
        let (handle, _sender) = create_room_task();
        let (socket, _) = create_participant_connection();
        let client_parameters = ClientParameters {
            device_secret: DeviceSecret::example_data(),
            kind: client_parameters::ClientKind::Guest {
                display_name: DisplayName::from_str_lossy("tester"),
            },
            role: client_parameters::Role::Moderator,
        };

        handle
            .accept_signaling_socket(socket, client_parameters)
            .await
            .unwrap();
    }
}
