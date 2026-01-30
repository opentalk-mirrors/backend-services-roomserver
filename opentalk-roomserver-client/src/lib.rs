// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! # RoomServer API Requests
//!
//! This crate provides the API requests to interact with the roomserver.

use api::signaling::{SignalingConnection, SignalingError};
use bytes::Bytes;
use http::{HeaderValue, StatusCode, header::InvalidHeaderValue};
use opentalk_roomserver_types::{
    api::{RoomServerAccess, TokenRequestBody},
    client_parameters::ClientParameters,
    room_parameters::RoomParameters,
};
use opentalk_service_auth::{ApiKey, EncodingError};
use opentalk_types_common::{rooms::RoomId, roomserver::Token};
use reqwest::{Client as ReqwestClient, Response, header::AUTHORIZATION};
use serde::Deserialize;
use thiserror::Error;
use url::Url;

pub mod api;

#[derive(Debug, Error)]
pub enum InvalidApiToken {
    #[error("Failed to encode JSON web token: {0:?}")]
    EncodingError(#[from] EncodingError),
    #[error("Failed to create authorization header {0:?} ")]
    ParsingError(#[from] InvalidHeaderValue),
}

#[derive(Debug, Error)]
pub enum Error<T> {
    #[error("Failed to set authorization header: {0}")]
    TokenError(#[from] InvalidApiToken),
    #[error("Failed to parse URL: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("RoomServer returned an error: {0:#?}")]
    ApiError(#[from] ApiError<T>),
    #[error("RoomServer returned an unexpected response:\nstatus: {status}\nbody: {body}")]
    Unexpected { status: StatusCode, body: String },
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
    InvalidApiToken,
    #[error("The room already exists, but is shutting down")]
    NotFound,
    #[error("An internal server error occurred")]
    InternalServerError,
}

#[derive(Error, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RequestTokenError {
    #[error("The provided API token is invalid")]
    InvalidApiToken,
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
    api_key: ApiKey,
}

impl Client {
    pub fn new(base_url: Url, api_key: ApiKey) -> Client {
        let reqwest_client = ReqwestClient::new();

        Self {
            reqwest_client,
            base_url,
            api_key,
        }
    }

    fn auth_header(&self) -> Result<HeaderValue, InvalidApiToken> {
        Ok(format!("Bearer {}", self.api_key.generate_jwt()?).parse()?)
    }

    #[tracing::instrument(skip(self, parameters))]
    pub async fn put_room(
        &self,
        room_id: RoomId,
        parameters: RoomParameters,
    ) -> Result<(), Error<PutRoomError>> {
        let url = self.base_url.join(&format!("v1/rooms/{room_id}"))?;
        let response = self
            .reqwest_client
            .put(url)
            .header(AUTHORIZATION, self.auth_header()?)
            .json(&parameters)
            .send()
            .await?;

        if response.status().is_success() {
            return Ok(());
        }

        Err(Self::parse_api_error::<PutRoomError>(
            response.status(),
            &response.bytes().await?,
        ))
    }

    #[tracing::instrument(skip(self, client_parameters, room_parameters))]
    pub async fn request_token(
        &self,
        room_id: RoomId,
        client_parameters: ClientParameters,
        room_parameters: Option<RoomParameters>,
    ) -> Result<RoomServerAccess, Error<RequestTokenError>> {
        let url = self.base_url.join(&format!("v1/rooms/{room_id}/token"))?;
        let response = self
            .reqwest_client
            .post(url)
            .header(AUTHORIZATION, &self.auth_header()?)
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
        let status = response.status();
        let body = response.bytes().await?;

        if status.is_success() {
            let result = serde_json::from_slice(&body).map_err(|_| {
                tracing::error!(
                    "Received unexpected response from RoomServer: {}",
                    String::from_utf8_lossy(&body)
                );
                Error::Unexpected {
                    status,
                    body: String::from_utf8_lossy(&body).into(),
                }
            })?;
            return Ok(result);
        }

        Err(Self::parse_api_error::<E>(status, &body))
    }

    fn parse_api_error<E>(status: StatusCode, body: &Bytes) -> Error<E>
    where
        E: for<'de> Deserialize<'de>,
    {
        match serde_json::from_slice::<ApiError<E>>(body) {
            Ok(err) => Error::ApiError(err),
            Err(_) => {
                tracing::error!(
                    "Received unexpected error response from RoomServer: {}",
                    String::from_utf8_lossy(body)
                );
                Error::Unexpected {
                    status,
                    body: String::from_utf8_lossy(body).into(),
                }
            }
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn open_signaling_connection(
        &self,
        url: Url,
        token: Token,
    ) -> Result<SignalingConnection, SignalingError> {
        SignalingConnection::connect(url, token).await
    }
}
