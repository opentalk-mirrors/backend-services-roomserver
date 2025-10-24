// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! # RoomServer API Requests
//!
//! This crate provides the API requests to interact with the roomserver.

use api::signaling::{SignalingConnection, SignalingError};
use bytes::Bytes;
use http::{HeaderValue, header::InvalidHeaderValue};
use opentalk_roomserver_types::{
    api::{RoomServerAccess, TokenRequestBody},
    client_parameters::ClientParameters,
    room_parameters::RoomParameters,
};
use opentalk_types_common::{rooms::RoomId, roomserver::Token};
use reqwest::{Client as ReqwestClient, Response, header::AUTHORIZATION};
use serde::Deserialize;
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

#[derive(Debug, Error)]
pub enum Error<T> {
    #[error("Failed to parse URL: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("RoomServer returned an error: {0:#?}")]
    ApiError(#[from] ApiError<T>),
    #[error("RoomServer returned an unexpected response")]
    Unexpected,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ApiError<T> {
    pub code: T,
    pub message: String,
}

#[derive(Error, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PutRoomError {
    #[error("The provided API token is invalid")]
    Unauthorized,
    #[error("The room already exists, but is shutting down")]
    NotFound,
    #[error("An internal server error occurred")]
    InternalServerError,
}

#[derive(Error, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RequestTokenError {
    #[error("The provided API token is invalid")]
    Unauthorized,
    #[error("The requested room does not exist and no room parameters were provided")]
    RoomParametersMissing,
    #[error("An internal server error occurred")]
    InternalServerError,
    #[error("The requesting participant is banned from the room")]
    Banned,
}

#[derive(Debug)]
pub struct Client {
    reqwest_client: ReqwestClient,
    base_url: Url,
    api_token: HeaderValue,
}

impl Client {
    pub fn new(base_url: Url, api_token: &str) -> Result<Client, InvalidApiTokenError> {
        let reqwest_client = ReqwestClient::new();

        let api_token = format!("bearer {api_token}").parse()?;
        Ok(Self {
            reqwest_client,
            base_url,
            api_token,
        })
    }

    pub async fn put_room(
        &self,
        room_id: RoomId,
        parameters: RoomParameters,
    ) -> Result<(), Error<PutRoomError>> {
        let url = self.base_url.join(&format!("/v1/rooms/{room_id}"))?;
        let response = self
            .reqwest_client
            .put(url)
            .header(AUTHORIZATION, self.api_token.clone())
            .json(&parameters)
            .send()
            .await?;

        if response.status().is_success() {
            return Ok(());
        }
        Err(Self::parse_api_error::<PutRoomError>(
            &response.bytes().await?,
        ))
    }

    pub async fn request_token(
        &self,
        room_id: RoomId,
        client_parameters: ClientParameters,
        room_parameters: Option<RoomParameters>,
    ) -> Result<RoomServerAccess, Error<RequestTokenError>> {
        let url = self.base_url.join(&format!("/v1/rooms/{room_id}/token"))?;
        let response = self
            .reqwest_client
            .post(url)
            .header(AUTHORIZATION, self.api_token.clone())
            .json(&TokenRequestBody {
                client_parameters,
                room_parameters,
            })
            .send()
            .await?;

        Self::parse_api_response(response).await
    }

    async fn parse_api_response<T, E>(response: Response) -> Result<T, Error<E>>
    where
        T: for<'de> Deserialize<'de>,
        E: for<'de> Deserialize<'de>,
    {
        let success = response.status().is_success();
        let bytes = response.bytes().await?;

        if success {
            let result = serde_json::from_slice(&bytes).map_err(|_| {
                log::error!(
                    "Received unexpected response from RoomServer: {}",
                    String::from_utf8_lossy(&bytes)
                );
                Error::Unexpected
            })?;
            return Ok(result);
        }

        Err(Self::parse_api_error::<E>(&bytes))
    }

    fn parse_api_error<E>(bytes: &Bytes) -> Error<E>
    where
        E: for<'de> Deserialize<'de>,
    {
        match serde_json::from_slice::<ApiError<E>>(bytes) {
            Ok(err) => Error::ApiError(err),
            Err(_) => {
                log::error!(
                    "Received unexpected error response from RoomServer: {}",
                    String::from_utf8_lossy(bytes)
                );
                Error::Unexpected
            }
        }
    }

    pub async fn open_signaling_connection(
        &self,
        url: Url,
        token: Token,
    ) -> Result<SignalingConnection, SignalingError> {
        SignalingConnection::connect(url, token).await
    }
}
