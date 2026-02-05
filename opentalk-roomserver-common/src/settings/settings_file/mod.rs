// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow};
use conference::Conference;
use config::{Config, Environment, File, FileFormat};
use defaults::Defaults;
use http::Http;
use opentalk_orchestrator_client::OrchestratorConfig;
use reports::Reports;
use serde::Deserialize;
use telemetry::{Metrics, Monitoring, Tracing};
use thiserror::Error;

pub mod conference;
pub mod defaults;
pub mod http;
pub mod livekit;
pub mod reports;
pub mod reports_typst;
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

    pub(crate) orchestrator: Option<OrchestratorConfig>,

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

    #[serde(default)]
    pub(crate) reports: Option<Reports>,
}

impl SettingsFile {
    fn environment() -> Environment {
        Environment::with_prefix("OT_ROOMSERVER")
            .prefix_separator("_")
            .separator("__")
            // try_parsing, list_separator and with_list_parse_key are required
            // to parse sequences from the environment
            .try_parsing(true)
            .list_separator(",")
            .with_list_parse_key("http.api_keys")
            .with_list_parse_key("metrics.allowlist")
    }

    /// Creates a new Settings instance from the provided TOML file.
    /// Specific fields can be set or overwritten with environment variables (See struct level docs
    /// for more details).
    pub fn load_from_path(file_path: &Path) -> Result<Self, Error> {
        let config = Config::builder()
            .add_source(File::from(file_path).format(FileFormat::Toml))
            .add_source(Self::environment())
            .build()
            .context("failed to build settings loader")?;

        Ok(serde_path_to_error::deserialize(config).context("invalid configuration")?)
    }

    /// Creates a new Settings instance from the provided TOML file if provided
    /// or from the first available standard path otherwise.
    /// Specific fields can be set or overwritten with environment variables (See struct level docs
    /// for more details).
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

#[cfg(test)]
mod tests {
    use config::Config;

    use super::SettingsFile;

    #[test]
    fn should_parse_array_from_env() {
        temp_env::with_vars(
            [
                (
                    "OT_ROOMSERVER_METRICS__ALLOWLIST",
                    Some("172.0.0.0/9,172.128.0.0/9"),
                ),
                (
                    "OT_ROOMSERVER_HTTP__API_KEYS",
                    Some("roomserver:secret1,recorder:secret2"),
                ),
            ],
            || {
                let environment = SettingsFile::environment();

                let config = Config::builder()
                    .add_source(environment)
                    .set_default("http.public_url", "http://localhost")
                    .unwrap()
                    .build()
                    .unwrap();

                config.try_deserialize::<SettingsFile>().unwrap();
            },
        );
    }

    #[test]
    fn should_parse_array_from_env_single() {
        temp_env::with_vars(
            [
                ("OT_ROOMSERVER_METRICS__ALLOWLIST", Some("172.0.0.0/9")),
                ("OT_ROOMSERVER_HTTP__API_KEYS", Some("roomserver:secret1")),
            ],
            || {
                let environment = SettingsFile::environment();

                let config = Config::builder()
                    .add_source(environment)
                    .set_default("http.public_url", "http://localhost")
                    .unwrap()
                    .build()
                    .unwrap();

                config.try_deserialize::<SettingsFile>().unwrap();
            },
        );
    }
}
