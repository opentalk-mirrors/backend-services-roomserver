// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use bytes::Bytes;
use http::StatusCode;
use http_request_derive::{FromHttpResponse, HttpRequest};
use opentalk_roomserver_types::{
    api::{TokenRequestBody, TokenResponse},
    room_parameters::RoomParameters,
};
use opentalk_types_common::rooms::RoomId;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

#[derive(HttpRequest)]
#[http_request(method = "PUT", response = RoomCreateResponse, path = "/rooms/{room_id}")]
pub(crate) struct RoomsCreateRequest {
    #[http_request(body)]
    pub(crate) body: RoomParameters,

    pub(crate) room_id: RoomId,

    #[http_request(header)]
    headers: HeaderMap,
}

impl RoomsCreateRequest {
    pub(crate) fn new(room_id: RoomId, parameter: RoomParameters, api_token: HeaderValue) -> Self {
        let mut headers: HeaderMap = HeaderMap::new();
        headers.append(AUTHORIZATION, api_token);

        Self {
            body: parameter,
            room_id,
            headers,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum RoomCreateResponse {
    Created,
    Updated,
}

impl FromHttpResponse for RoomCreateResponse {
    fn from_http_response(
        http_response: http::Response<Bytes>,
    ) -> Result<Self, http_request_derive::Error>
    where
        Self: Sized,
    {
        let status = http_response.status();
        if status == StatusCode::CREATED {
            Ok(Self::Created)
        } else if status == StatusCode::NO_CONTENT {
            Ok(Self::Updated)
        } else {
            Err(http_request_derive::Error::Custom {
                message: format!("invalid status code: {status}"),
                location: Default::default(),
            })
        }
    }
}

#[derive(HttpRequest)]
#[http_request(method = "POST", response = TokenResponse, path = "/rooms/{room_id}/token")]
pub(crate) struct TokenRequest {
    pub(crate) room_id: RoomId,

    #[http_request(header)]
    headers: HeaderMap,

    #[http_request(body)]
    pub(crate) body: TokenRequestBody,
}

impl TokenRequest {
    pub(crate) fn new(room_id: RoomId, body: TokenRequestBody, api_token: HeaderValue) -> Self {
        let mut headers = HeaderMap::new();
        headers.append(AUTHORIZATION, api_token);

        Self {
            room_id,
            headers,
            body,
        }
    }
}
