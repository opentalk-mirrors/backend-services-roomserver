// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{collections::BTreeSet, net::Ipv6Addr, sync::Arc};

use opentalk_roomserver_common::{
    application_state::ApplicationState,
    settings::{Http, Settings},
};
use opentalk_roomserver_signaling::signaling_module::SignalingModule;
use opentalk_roomserver_types::{
    client_parameters::ClientParameters, core_event::CoreEvent, room_parameters::RoomParameters,
};
use opentalk_types_common::{rooms::RoomId, tariffs::TariffModuleResource, utils::ExampleData};
use tokio::sync::watch;

use super::participant::{MockParticipantJoined, MockParticipantJoining, ReceiveError};
use crate::{
    ModuleRegistry, RoomTaskHandle, RoomTaskHandleError,
    mocking::{participant::create_participant_connection, socket::MockSocket},
    task::{RoomTask, fs_storage::FsStorage},
};

#[derive(Debug)]
pub enum Error {
    RoomHandle(RoomTaskHandleError<MockSocket>),
    Participant(ReceiveError),
}

impl From<RoomTaskHandleError<MockSocket>> for Error {
    fn from(error: RoomTaskHandleError<MockSocket>) -> Self {
        Self::RoomHandle(error)
    }
}

impl From<ReceiveError> for Error {
    fn from(error: ReceiveError) -> Self {
        Self::Participant(error)
    }
}

fn settings() -> Settings {
    Settings {
        http: Http {
            address: std::net::IpAddr::V6(Ipv6Addr::LOCALHOST),
            port: 11333,
            api_token: "Secret".to_string(),
            disable_openapi: true,
        },
        monitoring: Default::default(),
        metrics: Default::default(),
        tracing: Default::default(),
        conference: Default::default(),
        defaults: Default::default(),
    }
}

pub struct TestRoomBuilder {
    room_id: RoomId,
    settings: Settings,
    room_parameters: RoomParameters,
    module_registry: ModuleRegistry,
}

impl TestRoomBuilder {
    pub fn new() -> Self {
        Self {
            room_id: RoomId::from_u128(1),
            settings: settings(),
            room_parameters: RoomParameters::example_data(),
            module_registry: ModuleRegistry::new(),
        }
    }

    pub fn register_module<M: SignalingModule + 'static>(mut self) -> Self {
        self.module_registry.add_module::<M>();
        self.room_parameters.tariff.modules.insert(
            M::NAMESPACE,
            TariffModuleResource {
                features: BTreeSet::default(),
            },
        );
        self
    }

    pub fn room_id(mut self, room_id: RoomId) -> Self {
        self.room_id = room_id;
        self
    }

    pub fn room_parameters(mut self, room_parameters: RoomParameters) -> Self {
        self.room_parameters = room_parameters;
        self
    }

    pub fn spawn(self) -> TestRoom {
        TestRoom::spawn(
            self.room_id,
            self.room_parameters,
            self.module_registry,
            self.settings,
        )
    }
}

impl Default for TestRoomBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TestRoom {
    room_id: RoomId,
    room_handle: RoomTaskHandle<MockSocket>,
    _settings: Arc<Settings>,
    _app_state_tx: watch::Sender<ApplicationState>,
}

impl TestRoom {
    pub fn builder() -> TestRoomBuilder {
        TestRoomBuilder::new()
    }

    fn spawn(
        room_id: RoomId,
        room_parameters: RoomParameters,
        module_registry: ModuleRegistry,
        settings: Settings,
    ) -> Self {
        let settings = Arc::new(settings);
        let (app_state_tx, rx) = watch::channel(ApplicationState::Running);

        let quota = 5 * 1024u64.pow(3); // 5GiB
        let storage = FsStorage::new(quota, None).expect("Failed to create storage");
        let storage = Arc::new(storage);

        let (room_handle, _) = RoomTask::spawn(
            room_id,
            room_parameters.into(),
            Arc::new(module_registry),
            storage,
            Arc::clone(&settings),
            rx,
        );

        Self {
            room_id,
            room_handle,
            _settings: settings,
            _app_state_tx: app_state_tx,
        }
    }

    pub async fn join_participant(
        &mut self,
        client_parameters: ClientParameters,
    ) -> Result<MockParticipantJoined, Error> {
        let (socket, participant) = create_participant_connection();
        self.room_handle
            .accept_signaling_socket(socket, client_parameters)
            .await?;
        let participant = participant.join_success().await?;

        Ok(participant)
    }

    pub async fn join_alice_moderator(&mut self, device_number: usize) -> MockParticipantJoined {
        MockParticipantJoining::alice(device_number)
            .join(self)
            .await
            .unwrap()
    }

    pub async fn join_bob(&mut self, device_number: usize) -> MockParticipantJoined {
        MockParticipantJoining::bob(device_number)
            .join(self)
            .await
            .unwrap()
    }

    pub async fn join_charlie(&mut self, device_number: usize) -> MockParticipantJoined {
        MockParticipantJoining::charlie(device_number)
            .join(self)
            .await
            .unwrap()
    }

    pub async fn join_dave(&mut self, device_number: usize) -> MockParticipantJoined {
        MockParticipantJoining::dave(device_number)
            .join(self)
            .await
            .unwrap()
    }

    pub async fn join_gustav_guest(&mut self) -> MockParticipantJoined {
        MockParticipantJoining::gustav().join(self).await.unwrap()
    }

    pub fn id(&self) -> RoomId {
        self.room_id
    }
}

pub async fn flush_connected_events(others: &mut [&mut MockParticipantJoined]) {
    for p in others {
        assert!(matches!(
            p.receive::<CoreEvent>().await.unwrap().content,
            CoreEvent::ParticipantConnected { .. }
        ));
    }
}
