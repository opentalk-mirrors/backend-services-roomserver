// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::net::IpAddr;

use crate::settings::settings_file;

/// Settings for the HTTP server
#[derive(Debug, Clone)]
pub struct Http {
    /// The IP address that the HTTP server should bind to
    pub address: IpAddr,

    /// The port that the HTTP server should use
    pub port: u16,

    /// The publicly reachable URL of this server
    pub public_url: url::Url,

    /// The API token for service endpoints
    pub api_token: String,

    // Disable the OpenAPI endpoint under `/v1/openapi.json` and the corresponding
    // swagger endpoint under `/swagger`.
    pub disable_openapi: bool,
}

impl From<settings_file::http::Http> for Http {
    fn from(value: settings_file::http::Http) -> Self {
        Self {
            address: value.address,
            port: value.port,
            public_url: value.public_url,
            api_token: value.api_token,
            disable_openapi: value.disable_openapi,
        }
    }
}
