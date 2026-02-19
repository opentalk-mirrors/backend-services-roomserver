// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! Contains code for room management and signaling.
//!
//! The room state is managed by the [`task::RoomTask`], where each room has its own [`tokio::task`]
//! with an instance of a [`RoomTask`](task::RoomTask). The [`RoomTasks`](task::RoomTask) have a
//! channel interface that is exposed via the [`RoomTaskHandle`] through which the web api can send
//! requests to each individual room.
//!
//! The active rooms are created and tracked with the [`RoomTaskRegistry`]. When a
//! [`task::RoomTask`] gets destroyed, it removes itself from the [`RoomTaskRegistry`].

pub mod message_router;
pub mod metrics;
#[cfg(any(test, feature = "mock"))]
pub mod mocking;
pub mod orchestrator_metrics;
pub mod registry;
pub mod signaling;
pub mod storage;
pub mod task;

pub use opentalk_roomserver_signaling::storage::assets::{
    AssetMetaData, AssetUploaded, ModuleAssetStorage, StorageError,
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
    use std::{
        collections::{BTreeMap, BTreeSet},
        sync::Arc,
        time::Duration,
    };

    use opentalk_roomserver_common::{application_state::ApplicationState, settings::Settings};
    use opentalk_roomserver_types::{
        client_parameters::{self, ClientParameters, Role},
        core::{CoreEvent, JoinBlockedReason},
        room_parameters::RoomParameters,
        tariff_details::TariffDetails,
    };
    use opentalk_roomserver_web_api::v1::signaling::websocket::{
        CloseFrame, SignalingSocketMessage,
    };
    use opentalk_types_common::{
        rooms::RoomId,
        roomserver::DeviceSecret,
        tariffs::{QuotaType, TariffId},
        users::DisplayName,
        utils::ExampleData,
    };
    use tokio::{sync::watch, time::sleep};

    use super::{signaling::module_initializer::ModuleRegistry, task::handle::RoomTaskHandle};
    use crate::{
        mocking::{participant::create_participant_connection, room::TestRoom, socket::MockSocket},
        task::RoomTask,
    };

    const TIMEOUT: Duration = Duration::from_millis(500);

    fn create_room_task() -> (RoomTaskHandle<MockSocket>, watch::Sender<ApplicationState>) {
        let id = RoomId::from_u128(0xc270ab35_5cdb_4614_b872_8dd66ceefc70);
        let params = Arc::new(RoomParameters::example_data());
        let module_registry = Arc::new(ModuleRegistry::new());
        let (sender, state) = watch::channel(ApplicationState::Running);
        let settings = Arc::new(Settings::test_settings("secret".to_owned()));

        let (task_handle, future_room) =
            RoomTask::setup(id, params, module_registry, settings, state, TIMEOUT);
        tokio::spawn(future_room);
        (task_handle, sender)
    }
    #[test_log::test(tokio::test)]
    async fn timeout() {
        let (handle, _sender) = create_room_task();
        sleep(TIMEOUT - Duration::from_millis(100)).await;
        handle.refresh_idle_timeout().await.unwrap();
        sleep(TIMEOUT + Duration::from_millis(100)).await;
        handle.refresh_idle_timeout().await.unwrap_err();
    }

    #[test_log::test(tokio::test)]
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

    #[test_log::test(tokio::test)]
    async fn room_participant_limit() {
        let mut room = TestRoom::builder()
            .tariff(TariffDetails {
                id: TariffId::generate(),
                name: "Room Participant Limit".into(),
                quotas: BTreeMap::from_iter([(QuotaType::RoomParticipantLimit, 2)]),
                disabled_features: BTreeSet::new(),
            })
            .spawn();

        // Alice and Bob join the room, the room participant limit is now reached
        let _alice = room.join_alice_moderator(0).await;
        let _charlie = room.join_charlie(0).await;

        // Charlie tries to join the room, but is rejected because the room participant limit is
        // reached
        let (socket, mut participant) = create_participant_connection();
        room.room_handle
            .accept_signaling_socket(
                socket,
                ClientParameters {
                    device_secret: "Device Secret Charlie".parse().unwrap(),
                    kind: client_parameters::ClientKind::Guest {
                        display_name: "Charlie".parse().unwrap(),
                    },
                    role: Role::User,
                },
            )
            .await
            .unwrap();

        // Charlie first receives a `JoinBlocked` event
        let msg = participant.receiver.recv().await.unwrap();
        let SignalingSocketMessage::Text(msg) = msg else {
            panic!("Expected text message, received {msg:?}");
        };
        let event: CoreEvent = serde_json::from_str(&msg).unwrap();
        assert!(
            matches!(
                event,
                CoreEvent::JoinBlocked {
                    reason: JoinBlockedReason::ParticipantLimitReached
                },
            ),
            "Expected `CoreEvent`, received {event:#?}"
        );

        // Then Charlie receives a close frame with code 1013 (Try Again Later)
        let msg = participant.receiver.recv().await.unwrap();
        assert!(
            matches!(
                msg,
                SignalingSocketMessage::Close(Some(CloseFrame { code: 1013, .. }))
            ),
            "Expected close message, received {msg:?}"
        );
    }
}
