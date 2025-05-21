// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! # RoomServer API Requests
//!
//! This crate provides the API requests to interact with the roomserver.

use api::{
    room::{RoomsCreateRequest, TokenRequest},
    signaling::{SignalingConnection, SignalingError},
};
use http::{HeaderValue, header::InvalidHeaderValue};
use http_request_derive_client::Client as _;
use http_request_derive_client_reqwest::{ReqwestClient, ReqwestClientError};
use opentalk_roomserver_types::{
    api::{TokenRequestBody, TokenResponse},
    client_parameters::ClientParameters,
    room_parameters::RoomParameters,
};
use opentalk_types_common::{rooms::RoomId, roomserver::Token};
use thiserror::Error;
use url::Url;

pub mod api;

/// The API token is invalid
#[derive(Debug, Error)]
#[error("invalid API token")]
pub struct InvalidApiTokenError {
    #[from]
    source: InvalidHeaderValue,
}

/// The request to the RoomServer failed
#[derive(Debug, Error)]
#[error("request failed")]
pub struct ServerError {
    #[from]
    source: ReqwestClientError,
}

#[derive(Debug)]
pub struct Client {
    base_url: Url,
    reqwest_client: ReqwestClient,
    api_token: HeaderValue,
}

impl Client {
    pub fn new(base_url: Url, api_token: String) -> Result<Client, InvalidApiTokenError> {
        let reqwest_client = ReqwestClient::new(base_url.clone());

        let api_token = format!("bearer {api_token}").parse()?;
        Ok(Self {
            base_url,
            reqwest_client,
            api_token,
        })
    }

    pub async fn put_room(
        &self,
        room_id: RoomId,
        parameters: RoomParameters,
    ) -> Result<(), ServerError> {
        let _response = self
            .reqwest_client
            .execute(RoomsCreateRequest::new(
                room_id,
                parameters,
                self.api_token.clone(),
            ))
            .await?;

        Ok(())
    }

    pub async fn request_token(
        &self,
        room_id: RoomId,
        client_parameters: ClientParameters,
        room_parameters: Option<RoomParameters>,
    ) -> Result<Option<Token>, ServerError> {
        let request = TokenRequest::new(
            room_id,
            TokenRequestBody {
                client_parameters,
                room_parameters,
            },
            self.api_token.clone(),
        );
        let response = self.reqwest_client.execute(request).await?;
        match response {
            TokenResponse::Token { token } => Ok(Some(token)),
            TokenResponse::UnknownRoom => Ok(None),
        }
    }

    pub async fn open_signaling_connection(
        &self,
        token: Token,
    ) -> Result<SignalingConnection, SignalingError> {
        SignalingConnection::connect(self.base_url.clone(), token).await
    }
}
