// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::net::{IpAddr, Ipv4Addr};

use config::{Config, Environment, File, FileFormat};
use serde::Deserialize;
use telemetry::{Metrics, Monitoring, Tracing};

pub mod telemetry;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    /// HTTP web server settings
    pub(crate) http: Http,

    #[serde(default)]
    pub(crate) monitoring: Option<Monitoring>,

    #[serde(default)]
    pub(crate) metrics: Option<Metrics>,

    #[serde(default)]
    pub(crate) tracing: Option<Tracing>,
}

impl Settings {
    /// Creates a new Settings instance from the provided TOML file.
    /// Specific fields can be set or overwritten with environment variables (See struct level docs for more details).
    pub(crate) fn load(file_name: &str) -> anyhow::Result<Self> {
        let config = Config::builder()
            .add_source(File::new(file_name, FileFormat::Toml))
            .add_source(
                Environment::with_prefix("OT_ROOMSERVER")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?;

        Ok(serde_path_to_error::deserialize(config)?)
    }

    #[cfg(test)]
    pub(crate) fn test_settings(api_token: String) -> Self {
        Self {
            http: Http {
                address: default_bind_address(),
                port: default_port(),
                api_token,
                disable_openapi: false,
            },
            monitoring: None,
            metrics: None,
            tracing: None,
        }
    }
}

/// Settings for the HTTP server
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Http {
    /// The IP address that the HTTP server should bind to
    #[serde(default = "default_bind_address")]
    pub(crate) address: IpAddr,

    /// The port that the HTTP server should use
    #[serde(default = "default_port")]
    pub(crate) port: u16,

    /// The API token for service endpoints
    pub(crate) api_token: String,

    // Disable the OpenAPI endpoint under `/v1/openapi.json` and the corresponding
    // swagger endpoint under `/swagger`.
    #[serde(default)]
    pub(crate) disable_openapi: bool,
}

fn default_bind_address() -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
}

const fn default_port() -> u16 {
    11333
}
