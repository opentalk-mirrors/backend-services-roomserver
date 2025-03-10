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
pub(crate) mod task;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use opentalk_roomserver_types::room_parameters::RoomParameters;
    use opentalk_types_api_v1::users::PublicUserProfile;
    use opentalk_types_common::{rooms::RoomId, tariffs::TariffResource, utils::ExampleData};
    use tokio::{sync::watch, time::sleep};

    use super::task::handle::RoomTaskHandle;
    use crate::{
        mocking::{mock_socket::MockSocket, participant::create_participant_connection},
        room::{registry::RoomTaskRegistry, task::RoomTask},
        ApplicationState,
    };

    const TIMEOUT: Duration = Duration::from_millis(500);

    fn create_room_parameters() -> RoomParameters {
        RoomParameters {
            created_by: PublicUserProfile::example_data(),
            password: Default::default(),
            waiting_room: Default::default(),
            call_in: Default::default(),
            event: Default::default(),
            invite_code: Default::default(),
            tariff: TariffResource::example_data(),
            streaming_links: Default::default(),
        }
    }

    fn create_room_task() -> RoomTaskHandle<MockSocket> {
        let id = RoomId::from_u128(0xc270ab35_5cdb_4614_b872_8dd66ceefc70);
        let params = create_room_parameters();
        let registry = RoomTaskRegistry::new();
        let (_, state) = watch::channel(ApplicationState::Running);
        RoomTask::spawn_with_timeout(id, params, registry, state, TIMEOUT)
    }

    #[tokio::test]
    async fn timeout() {
        let handle = create_room_task();
        sleep(TIMEOUT - Duration::from_millis(100)).await;
        handle.refresh_idle_timeout().await.unwrap();
        sleep(TIMEOUT + Duration::from_millis(100)).await;
        handle.refresh_idle_timeout().await.unwrap_err();
    }

    #[tokio::test]
    async fn accept_signaling_socket() {
        let handle = create_room_task();
        let (socket, _) = create_participant_connection();
        handle.accept_signaling_socket(socket).await.unwrap();
    }
}
