// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! # RoomServer API Requests
//!
//! This crate provides the API requests to interact with the roomserver.

use anyhow::Context;
use api::{
    room::{RoomsCreateRequest, TokenRequest},
    signaling::SignalingConnection,
};
use http_request_derive_client::Client as _;
use http_request_derive_client_reqwest::ReqwestClient;
use opentalk_roomserver_types::{
    api::{TokenRequestBody, TokenResponse},
    client_parameters::ClientParameters,
    room_parameters::RoomParameters,
};
use opentalk_types_common::{rooms::RoomId, roomserver::Token};
use url::Url;

pub mod api;

pub struct Client {
    base_url: Url,
    reqwest_client: ReqwestClient,
    api_token: String,
}

impl Client {
    pub fn new(base_url: Url, api_token: String) -> Client {
        let reqwest_client = ReqwestClient::new(base_url.clone());

        Self {
            base_url,
            reqwest_client,
            api_token: format!("bearer {api_token}"),
        }
    }

    pub async fn put_room(
        &self,
        room_id: RoomId,
        parameters: RoomParameters,
    ) -> anyhow::Result<()> {
        let response = self
            .reqwest_client
            .execute(RoomsCreateRequest::new(
                room_id,
                parameters,
                self.api_token.parse().context("Invalid api_token")?,
            ))
            .await;

        match response {
            Ok(_) => Ok(()),
            Err(e) => Err(e).context("received error response"),
        }
    }

    pub async fn request_token(
        &self,
        room_id: RoomId,
        client_parameters: ClientParameters,
        room_parameters: Option<RoomParameters>,
    ) -> anyhow::Result<Option<Token>> {
        let request = TokenRequest::new(
            room_id,
            TokenRequestBody {
                client_parameters,
                room_parameters,
            },
            self.api_token.parse().context("Invalid api_token")?,
        );
        let response = self
            .reqwest_client
            .execute(request)
            .await
            .context("token request failed")?;
        match response {
            TokenResponse::Token { token } => Ok(Some(token)),
            TokenResponse::UnknownRoom => Ok(None),
        }
    }

    pub async fn open_signaling_connection(
        &self,
        token: Token,
    ) -> anyhow::Result<SignalingConnection> {
        SignalingConnection::connect(self.base_url.clone(), token).await
    }
}
