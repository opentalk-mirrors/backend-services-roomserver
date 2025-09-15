// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_module_whiteboard::WhiteboardModule;
use opentalk_roomserver_room::mocking::room::TestRoom;
use opentalk_roomserver_types_whiteboard::WhiteboardSettings;
use opentalk_types_common::rooms::RoomId;
use reqwest::Url;
use testcontainers::{
    ContainerAsync, GenericImage, ImageExt as _,
    core::{IntoContainerPort as _, WaitFor, logs::consumer::logging_consumer::LoggingConsumer},
    runners::AsyncRunner,
};

pub const SPACEDECK_PORT: u16 = 9666;
pub const SPACEDECK_API_KEY: &str = "secret123";

const ENV_SPACEDECK_HOST: &str = "TEST_ROOMSERVER_SPACEDECK_HOST";
const ENV_SPACEDECK_PORT: &str = "TEST_ROOMSERVER_SPACEDECK_PORT";
const ENV_SPACEDECK_API_KEY: &str = "SD_API_TOKEN";

pub async fn build_whiteboard_room() -> (Option<ContainerAsync<GenericImage>>, TestRoom) {
    if let Ok(host) = std::env::var(ENV_SPACEDECK_HOST) {
        let room = build_room_from_env(&host);
        (None, room)
    } else {
        let (container, room) = build_room_from_test_container().await;
        (Some(container), room)
    }
}

fn build_room_from_env(host: &str) -> TestRoom {
    let port = std::env::var(ENV_SPACEDECK_PORT)
        .unwrap()
        .parse()
        .unwrap_or_else(|_| panic!("Environment variable {ENV_SPACEDECK_PORT} not valid"));
    let api_key = std::env::var(ENV_SPACEDECK_API_KEY).unwrap();
    build_room(host, port, api_key)
}

async fn build_room_from_test_container() -> (ContainerAsync<GenericImage>, TestRoom) {
    let spacedeck_container =
        GenericImage::new("registry.opencode.de/opentalk/spacedeck", "latest")
            .with_wait_for(WaitFor::message_on_stdout("created controller user"))
            .with_mapped_port(0, SPACEDECK_PORT.tcp())
            .with_env_var(ENV_SPACEDECK_API_KEY, SPACEDECK_API_KEY)
            .with_network("bridge")
            .with_log_consumer(LoggingConsumer::new())
            .start()
            .await
            .unwrap();

    let host = spacedeck_container.get_host().await.unwrap();
    let port = spacedeck_container
        .get_host_port_ipv4(SPACEDECK_PORT)
        .await
        .unwrap();

    let room = build_room(&host.to_string(), port, SPACEDECK_API_KEY.to_string());

    (spacedeck_container, room)
}

fn build_room(host: &str, port: u16, api_key: String) -> TestRoom {
    let base_url = Url::parse(&format!("http://{host}:{port}")).unwrap();
    let room_id = RoomId::generate();
    TestRoom::builder()
        .room_id(room_id)
        .register_module::<WhiteboardModule>()
        .add_init_module_data(&WhiteboardSettings { base_url, api_key })
        .unwrap()
        .spawn()
}
