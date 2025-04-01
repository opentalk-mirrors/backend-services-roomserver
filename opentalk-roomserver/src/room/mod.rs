// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! Contains code for room management and signaling.
//!
//! The room state is managed by the [`task::RoomTask`], where each room has its own [`tokio::task`] with an instance of
//! a [`RoomTask`](task::RoomTask). The [`RoomTasks`](task::RoomTask) have a channel interface that is exposed via the
//! [`RoomTaskHandle`](task::handle::RoomTaskHandle) through which the web api can send requests to each
//! individual room.
//!
//! The active rooms are created and tracked with the [`RoomTaskRegistry`](registry::RoomTaskRegistry). When a
//! [`task::RoomTask`] gets destroyed, it removes itself from the [`RoomTaskRegistry`](registry::RoomTaskRegistry).

mod message_router;
pub(crate) mod registry;
pub mod signaling;
pub(crate) mod task;

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use opentalk_roomserver_types::{
        client_parameters::{self, ClientParameters},
        room_parameters::RoomParameters,
    };
    use opentalk_types_common::{rooms::RoomId, users::DisplayName, utils::ExampleData};
    use tokio::{sync::watch, time::sleep};

    use super::{signaling::module_initializer::ModuleRegistry, task::handle::RoomTaskHandle};
    use crate::{
        mocking::{mock_socket::MockSocket, participant::create_participant_connection},
        room::{registry::RoomTaskRegistry, task::RoomTask},
        settings::Settings,
        ApplicationState,
    };

    const TIMEOUT: Duration = Duration::from_millis(500);

    fn create_room_task() -> (RoomTaskHandle<MockSocket>, watch::Sender<ApplicationState>) {
        let id = RoomId::from_u128(0xc270ab35_5cdb_4614_b872_8dd66ceefc70);
        let params = RoomParameters::example_data();
        let task_registry = RoomTaskRegistry::new();
        let module_registry = Arc::new(ModuleRegistry::new());
        let (sender, state) = watch::channel(ApplicationState::Running);
        let settings = Arc::new(Settings::test_settings("secret".to_owned()));
        (
            RoomTask::spawn_with_timeout(
                id,
                params,
                task_registry,
                state,
                module_registry,
                settings,
                TIMEOUT,
            ),
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
            client_id: "1234".into(),
            kind: client_parameters::ClientKind::Guest {
                display_name: DisplayName::from_str_lossy("tester"),
            },
        };

        handle
            .accept_signaling_socket(socket, client_parameters)
            .await
            .unwrap();
    }
}
