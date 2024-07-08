// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use anyhow::Result;
use config::{Config, Environment, File, FileFormat};
use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
pub(crate) struct Settings {
    /// HTTP web server settings
    #[serde(default)]
    pub(crate) http: Http,
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
    pub(crate) address: String,

    /// The port that the HTTP server should use
    #[serde(default = "default_port")]
    pub(crate) port: u16,
}

impl Default for Http {
    fn default() -> Self {
        Self {
            address: default_bind_address(),
            port: default_port(),
        }
    }
}

fn default_bind_address() -> String {
    "0.0.0.0".into()
}

fn default_port() -> u16 {
    11333
}
