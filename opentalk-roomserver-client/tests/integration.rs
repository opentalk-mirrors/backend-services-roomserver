// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

// These tests can unfortunately not be run in the CI. They require an up-to-date roomserver
// container to reflect the latest API. Running them in the CI would require building such a
// container before the tests, otherwise changes to the roomserver API could break the tests.

use opentalk_roomserver_client::{Client, Error, RequestTokenError};
use opentalk_roomserver_types::{
    client_parameters::ClientParameters, room_parameters::RoomParameters,
};
use opentalk_types_common::{rooms::RoomId, utils::ExampleData};
use testcontainers::{
    ContainerAsync, GenericImage, ImageExt,
    core::{IntoContainerPort, Mount, WaitFor, logs::consumer::logging_consumer::LoggingConsumer},
    runners::AsyncRunner,
};
use url::Url;

const ROOMSERVER_PORT: u16 = 11333;
const ROOMSERVER_API_TOKEN: &str = "secret";

#[test_log::test(tokio::test)]
#[ignore = "Requires an up-to-date roomserver container"]
async fn put_room() {
    let (_container, base_url) = spawn_roomserver().await;
    let client = Client::new(base_url, ROOMSERVER_API_TOKEN).unwrap();

    client
        .put_room(RoomId::from_u128(0x1), RoomParameters::example_data())
        .await
        .unwrap();
}

#[test_log::test(tokio::test)]
#[ignore = "Requires an up-to-date roomserver container"]
async fn request_token() {
    let (_container, base_url) = spawn_roomserver().await;
    let client = Client::new(base_url, ROOMSERVER_API_TOKEN).unwrap();

    let access = client
        .request_token(
            RoomId::from_u128(0x1),
            ClientParameters::example_data(),
            Some(RoomParameters::example_data()),
        )
        .await
        .unwrap();

    // Requesting a new token yields a different token each time
    let access2 = client
        .request_token(
            RoomId::from_u128(0x1),
            ClientParameters::example_data(),
            Some(RoomParameters::example_data()),
        )
        .await
        .unwrap();

    assert_ne!(access.token, access2.token);
}

#[test_log::test(tokio::test)]
#[ignore = "Requires an up-to-date roomserver container"]
async fn request_token_without_room_params() {
    let (_container, base_url) = spawn_roomserver().await;
    let client = Client::new(base_url, ROOMSERVER_API_TOKEN).unwrap();

    let error = client
        .request_token(
            RoomId::from_u128(0x1),
            ClientParameters::example_data(),
            None,
        )
        .await
        .unwrap_err();

    let Error::ApiError(api_error) = error else {
        panic!("Expected ApiError, received {error:#?}");
    };

    assert_eq!(api_error.code, RequestTokenError::RoomParametersMissing);
}

async fn spawn_roomserver() -> (ContainerAsync<GenericImage>, Url) {
    let config = std::env::current_dir()
        .unwrap()
        .join("../example/roomserver.toml");

    let container = GenericImage::new(
        "git.opentalk.dev:5050/opentalk/backend/services/roomserver",
        "latest",
    )
    .with_wait_for(WaitFor::message_on_stdout("Listening on"))
    .with_exposed_port(ROOMSERVER_PORT.tcp())
    .with_network("bridge")
    .with_mount(Mount::bind_mount(
        config.to_string_lossy(),
        "/etc/opentalk/roomserver.toml",
    ))
    .with_log_consumer(LoggingConsumer::new())
    .start()
    .await
    .unwrap();

    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(ROOMSERVER_PORT).await.unwrap();
    let base_url = format!("http://{host}:{port}");

    (container, base_url.parse().unwrap())
}
