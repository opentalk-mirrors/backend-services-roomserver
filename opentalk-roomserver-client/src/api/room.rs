// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use http_request_derive::HttpRequest;
use opentalk_roomserver_types::room_parameters::RoomParameters;
use opentalk_types::core::RoomId;
use serde::Deserialize;

#[derive(HttpRequest)]
#[http_request(method = "POST", response = RoomCreateResponse, path = "/rooms/create")]
pub struct RoomsCreateRequest {
    #[http_request(body)]
    pub body: RoomParameters,
}

#[derive(Deserialize, Debug)]
pub struct RoomCreateResponse;

#[derive(HttpRequest)]
#[http_request(method = "GET", response = String, path = "/rooms/probe/{room_id}")]
pub struct RoomsProbeRequest {
    pub room_id: RoomId,
}
