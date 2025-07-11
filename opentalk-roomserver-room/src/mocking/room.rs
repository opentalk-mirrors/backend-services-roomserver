// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::{BTreeMap, BTreeSet},
    net::Ipv6Addr,
    path::PathBuf,
    sync::Arc,
};

use anyhow::Context as _;
use opentalk_roomserver_common::{
    application_state::ApplicationState,
    settings::{Http, Settings},
};
use opentalk_roomserver_signaling::signaling_module::SignalingModule;
use opentalk_roomserver_types::{
    client_parameters::ClientParameters,
    core::CoreEvent,
    room_parameters::{EventContext, RoomParameters},
};
use opentalk_types_common::{
    rooms::RoomId,
    tariffs::{TariffId, TariffModuleResource, TariffResource},
};
use opentalk_types_signaling::{ModuleData, SignalingModuleFrontendData};
use tokio::sync::watch;

use super::participant::{MockParticipantJoined, MockParticipantJoining, ReceiveError};
use crate::{
    ModuleRegistry, RoomTaskHandle, RoomTaskHandleError,
    mocking::{
        participant::{
            MockParticipantWaiting, alice_public_profile, create_participant_connection,
        },
        socket::MockSocket,
    },
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
    storage_quota: u64,
}

impl TestRoomBuilder {
    pub fn new() -> Self {
        Self {
            room_id: RoomId::from_u128(1),
            settings: settings(),
            room_parameters: RoomParameters {
                created_by: alice_public_profile(),
                password: None,
                waiting_room: false,
                call_in: None,
                event: None,
                invite_code: None,
                tariff: TariffResource {
                    id: TariffId::from_u128(1),
                    name: "Default Tariff".to_string(),
                    quotas: BTreeMap::default(),
                    modules: BTreeMap::default(),
                },
                streaming_links: Vec::new(),
                e2e_encryption: false,
                module_data: ModuleData::new(),
            },
            module_registry: ModuleRegistry::new(),
            storage_quota: 5 * 1024u64.pow(3), // 5GiB
        }
    }

    pub fn event(mut self, event: EventContext) -> Self {
        self.room_parameters.event = Some(event);
        self
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

    pub fn add_init_module_data<T: SignalingModuleFrontendData>(
        mut self,
        data: &T,
    ) -> Result<Self, serde_json::Error> {
        self.room_parameters.module_data.insert(data)?;
        Ok(self)
    }

    pub fn waiting_room(mut self, enabled: bool) -> Self {
        self.room_parameters.waiting_room = enabled;
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

    pub fn storage_quota(mut self, quota: u64) -> Self {
        self.storage_quota = quota;
        self
    }

    pub fn spawn(self) -> TestRoom {
        TestRoom::spawn(
            self.room_id,
            self.room_parameters,
            self.module_registry,
            self.settings,
            self.storage_quota,
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
    storage: Arc<FsStorage>,
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
        storage_quota: u64,
    ) -> Self {
        let settings = Arc::new(settings);
        let (app_state_tx, rx) = watch::channel(ApplicationState::Running);

        let storage = FsStorage::new(storage_quota, None).expect("Failed to create storage");
        let storage = Arc::new(storage);
        let tmp = Arc::clone(&storage);

        let (room_handle, _) = RoomTask::spawn(
            room_id,
            room_parameters.into(),
            Arc::new(module_registry),
            tmp,
            Arc::clone(&settings),
            rx,
        );

        Self {
            room_id,
            room_handle,
            storage,
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

    pub async fn enter_waiting_room(
        &mut self,
        client_parameters: ClientParameters,
    ) -> Result<MockParticipantWaiting, Error> {
        let (socket, participant) = create_participant_connection();
        self.room_handle
            .accept_signaling_socket(socket, client_parameters)
            .await?;
        let participant = participant.join_waiting_room().await?;

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

    pub async fn waiting_room_bob(&mut self, device_number: usize) -> MockParticipantWaiting {
        MockParticipantJoining::bob(device_number)
            .enter_waiting_room(self)
            .await
            .unwrap()
    }

    pub async fn waiting_room_charlie(&mut self, device_number: usize) -> MockParticipantWaiting {
        MockParticipantJoining::charlie(device_number)
            .enter_waiting_room(self)
            .await
            .unwrap()
    }

    pub async fn waiting_room_dave(&mut self, device_number: usize) -> MockParticipantWaiting {
        MockParticipantJoining::dave(device_number)
            .enter_waiting_room(self)
            .await
            .unwrap()
    }

    pub async fn waiting_room_gustav_guest(&mut self) -> MockParticipantWaiting {
        MockParticipantJoining::gustav()
            .enter_waiting_room(self)
            .await
            .unwrap()
    }

    pub fn id(&self) -> RoomId {
        self.room_id
    }

    pub fn stored_files(&self) -> Vec<PathBuf> {
        self.storage.paths()
    }
}

pub async fn flush_connected_events(others: &mut [&mut MockParticipantJoined]) {
    for p in others {
        let event = p
            .receive::<CoreEvent>()
            .await
            .with_context(|| {
                format!(
                    "`{}` didn't receive an event",
                    p.join_success().display_name
                )
            })
            .unwrap();
        assert!(
            matches!(event.payload, CoreEvent::ParticipantConnected { .. },),
            "Participant `{}` didn't receive CoreEvent::ParticipantConnected",
            p.join_success().display_name.as_str()
        );
    }
}
