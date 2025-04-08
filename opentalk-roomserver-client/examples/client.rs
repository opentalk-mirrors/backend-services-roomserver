// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use anyhow::Context;
use opentalk_roomserver_client::Client;
use opentalk_roomserver_types::{
    client_parameters::{ClientKind, ClientParameters},
    room_parameters::RoomParameters,
};
use opentalk_types_common::{
    modules::ModuleId,
    rooms::RoomId,
    tariffs::{TariffModuleResource, TariffResource},
    utils::ExampleData,
};

const DEFAULT_ROOM_ID: RoomId = RoomId::from_u128(0x00000000_0000_0000_0000_000000000001);
const DEFAULT_HOST: &str = "http://localhost:11333";
const DEFAULT_API_TOKEN: &str = "secret";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    log::info!("RoomServer Client Example 🚀");
    let host = std::env::var("RS_CLIENT_HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string());
    let api_token =
        std::env::var("RS_CLIENT_API_TOKEN").unwrap_or_else(|_| DEFAULT_API_TOKEN.to_string());
    let client = Client::new(host.parse().context("Invalid roomserver host")?, api_token);

    let room_id = room_id_from_env()?.unwrap_or(DEFAULT_ROOM_ID);

    client
        .put_room(room_id, room_parameters())
        .await
        .context("Put roomserver failed")?;

    let token = client
        .request_token(
            room_id,
            ClientParameters {
                client_id: "example_client".to_string(),
                kind: ClientKind::Guest {
                    display_name: "Max Muster".parse().unwrap(),
                },
            },
            None,
        )
        .await
        .context("Failed to request room token")?;

    log::info!("Received room token: {token:?}");

    let _signaling_connection = client.open_signaling_connection(token).await?;

    log::info!("Signaling connection open");

    Ok(())
}

fn room_id_from_env() -> anyhow::Result<Option<RoomId>> {
    let Ok(room_id) = std::env::var("RS_CLIENT_ROOM_ID") else {
        return Ok(None);
    };
    let room_id = room_id
        .parse()
        .context("invalid room id provided via `RS_CLIENT_ROOM_ID` environment variable")?;

    Ok(Some(room_id))
}

fn room_parameters() -> RoomParameters {
    RoomParameters {
        tariff: TariffResource {
            modules: [("ping", TariffModuleResource::default())]
                .into_iter()
                .map(|(module, resource)| {
                    (
                        module.parse::<ModuleId>().expect("valid module id"),
                        resource,
                    )
                })
                .collect(),
            ..TariffResource::example_data()
        },
        ..RoomParameters::example_data()
    }
}
