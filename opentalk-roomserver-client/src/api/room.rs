// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use http_request_derive::HttpRequest;
use opentalk_roomserver_types::{
    api::{TokenRequestBody, TokenResponse},
    room_parameters::RoomParameters,
};
use opentalk_types_common::rooms::RoomId;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};

#[derive(HttpRequest)]
#[http_request(method = "PUT", response = RoomCreateResponse, path = "/rooms/{room_id}")]
pub struct RoomsCreateRequest {
    #[http_request(body)]
    pub body: RoomParameters,

    pub room_id: RoomId,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RoomCreateResponse;

#[derive(HttpRequest)]
#[http_request(method = "GET", response = String, path = "/rooms/{room_id}/probe")]
pub struct RoomsProbeRequest {
    pub room_id: RoomId,
}

#[derive(HttpRequest)]
#[http_request(method = "POST", response = TokenResponse, path = "/rooms/{room_id}/token")]
pub struct TokenRequest {
    pub room_id: RoomId,

    #[http_request(header)]
    headers: HeaderMap,

    #[http_request(body)]
    pub body: TokenRequestBody,
}

impl TokenRequest {
    pub fn new(room_id: RoomId, body: TokenRequestBody, api_token: HeaderValue) -> Self {
        let mut headers = HeaderMap::new();
        headers.append(AUTHORIZATION, api_token);

        Self {
            room_id,
            headers,
            body,
        }
    }
}
