// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow};
use conference::Conference;
use config::{Config, Environment, File, FileFormat};
use defaults::Defaults;
use http::Http;
use serde::Deserialize;
use telemetry::{Metrics, Monitoring, Tracing};
use thiserror::Error;

pub mod conference;
pub mod defaults;
pub mod http;
pub mod livekit;
pub mod telemetry;

#[derive(Debug, Error)]
#[error("Settings error")]
pub struct Error {
    #[from]
    source: anyhow::Error,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsFile {
    /// HTTP web server settings
    pub(crate) http: Http,

    #[serde(default)]
    pub(crate) monitoring: Option<Monitoring>,

    #[serde(default)]
    pub(crate) metrics: Option<Metrics>,

    #[serde(default)]
    pub(crate) tracing: Option<Tracing>,

    #[serde(default)]
    pub(crate) conference: Conference,

    #[serde(default)]
    pub(crate) defaults: Option<Defaults>,
}

impl SettingsFile {
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
    pub fn load(file_path: Option<&Path>) -> Result<SettingsFile, Error> {
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
}
