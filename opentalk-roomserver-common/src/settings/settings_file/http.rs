// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::net::{IpAddr, Ipv4Addr};

use serde::Deserialize;

/// Settings for the HTTP server
#[derive(Debug, Clone, Deserialize)]
pub struct Http {
    /// The IP address that the HTTP server should bind to
    #[serde(default = "default_bind_address")]
    pub address: IpAddr,

    /// The port that the HTTP server should use
    #[serde(default = "default_port")]
    pub port: u16,

    /// The API token for service endpoints
    pub api_token: String,

    // Disable the OpenAPI endpoint under `/v1/openapi.json` and the corresponding
    // swagger endpoint under `/swagger`.
    #[serde(default)]
    pub disable_openapi: bool,
}

pub(crate) fn default_bind_address() -> IpAddr {
    IpAddr::V4(Ipv4Addr::UNSPECIFIED)
}

const fn default_port() -> u16 {
    11333
}
