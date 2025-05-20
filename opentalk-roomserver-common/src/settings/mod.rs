// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    net::{IpAddr, Ipv4Addr},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use config::{Config, Environment, File, FileFormat};
use serde::Deserialize;
use signaling_salt::SignalingSalt;
use telemetry::{Metrics, Monitoring, Tracing};
use thiserror::Error;

pub mod signaling_salt;
pub mod telemetry;

#[derive(Debug, Error)]
#[error("Settings error")]
pub struct Error {
    #[from]
    source: anyhow::Error,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    /// HTTP web server settings
    pub http: Http,

    #[serde(default)]
    pub monitoring: Option<Monitoring>,

    #[serde(default)]
    pub metrics: Option<Metrics>,

    #[serde(default)]
    pub tracing: Option<Tracing>,

    #[serde(default)]
    pub conference: Conference,
}

impl Settings {
    /// Creates a new Settings instance from the provided TOML file.
    /// Specific fields can be set or overwritten with environment variables (See struct level docs for more details).
    pub fn load_from_path(file_path: &Path) -> Result<Self, Error> {
        let config = Config::builder()
            .add_source(File::from(file_path).format(FileFormat::Toml))
            .add_source(
                Environment::with_prefix("OT_ROOMSERVER")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()
            .context("failed to build settings loader")?;

        Ok(serde_path_to_error::deserialize(config).context("invalid configuration")?)
    }

    /// Creates a new Settings instance from the provided TOML file if provided
    /// or from the first available standard path otherwise.
    /// Specific fields can be set or overwritten with environment variables (See struct level docs for more details).
    pub fn load(file_path: Option<&Path>) -> Result<Settings, Error> {
        if let Some(path) = file_path {
            return Self::load_from_path(path);
        }

        let paths = Self::build_standard_search_paths();
        for path in &paths {
            if path.exists() {
                return Self::load_from_path(path);
            }
        }

        Err(anyhow!(
            "Couldn't find a configuration file. Searched: {}.",
            paths
                .iter()
                .map(|path| format!("\"{}\"", path.to_string_lossy()))
                .collect::<Vec<String>>()
                .join(", ")
        )
        .into())
    }

    fn build_standard_search_paths() -> Vec<PathBuf> {
        let mut paths = vec!["roomserver.toml".into()];

        if let Some(config_dir) = dirs::config_dir() {
            paths.push(config_dir.join("opentalk/roomserver.toml"));
        }

        paths.push("/etc/opentalk/roomserver.toml".into());

        paths
    }

    /// Creates settings for testing
    ///
    /// Do not use in production
    pub fn test_settings(api_token: String) -> Settings {
        Settings {
            http: Http {
                address: default_bind_address(),
                port: default_port(),
                api_token,
                disable_openapi: false,
            },
            monitoring: None,
            metrics: None,
            tracing: None,
            conference: Conference {
                signaling_salt: SignalingSalt("abcdefghijklmnopqrstuvwx".into()),
            },
        }
    }
}

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

fn default_bind_address() -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
}

const fn default_port() -> u16 {
    11333
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Conference {
    #[serde(default)]
    pub signaling_salt: SignalingSalt,
}
