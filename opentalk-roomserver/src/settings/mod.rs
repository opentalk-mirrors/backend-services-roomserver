// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::net::{IpAddr, Ipv4Addr};

use anyhow::Result;
use config::{Config, Environment, File, FileFormat};
use serde::Deserialize;
use telemetry::{Monitoring, Tracing};

pub mod telemetry;

#[derive(Debug, Default, Clone, Deserialize)]
pub(crate) struct Settings {
    /// HTTP web server settings
    #[serde(default)]
    pub(crate) http: Http,

    #[serde(default)]
    pub(crate) monitoring: Option<Monitoring>,

    #[serde(default)]
    pub(crate) tracing: Option<Tracing>,
}

impl Settings {
    /// Creates a new Settings instance from the provided TOML file.
    /// Specific fields can be set or overwritten with environment variables (See struct level docs for more details).
    pub(crate) fn load(file_name: &str) -> Result<Self> {
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

    // Disable the OpenAPI endpoint under `/v1/openapi.json` and the corresponding
    // swagger endpoint under `/swagger`.
    #[serde(default)]
    pub(crate) disable_openapi: bool,
}

impl Default for Http {
    fn default() -> Self {
        Self {
            address: default_bind_address(),
            port: default_port(),
            disable_openapi: false,
        }
    }
}

fn default_bind_address() -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
}

const fn default_port() -> u16 {
    11333
}
