// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use conference::Conference;
use defaults::Defaults;
use http::Http;
use internal::Internal;
use opentalk_orchestrator_client::OrchestratorConfig;
use opentalk_service_auth::{ApiKey, service::ApiKeys};
use reports::Reports;
use telemetry::{Metrics, Monitoring, Tracing};
use url::Url;

use super::{
    controller_settings::ControllerConfig, settings_file::SettingsFile,
    signaling_salt::SignalingSalt,
};
use crate::settings::runtime_settings::recording::Recording;

pub mod conference;
pub mod defaults;
pub mod http;
pub mod internal;
pub mod recording;
pub mod reports;
pub mod reports_typst;
pub mod telemetry;

#[derive(Debug, Clone)]
pub struct Settings {
    /// HTTP web server settings
    pub http: Http,

    pub controller: Option<ControllerConfig>,

    pub orchestrator: Option<OrchestratorConfig>,

    pub monitoring: Option<Monitoring>,

    pub metrics: Option<Metrics>,

    pub tracing: Option<Tracing>,

    pub conference: Conference,

    pub defaults: Option<Defaults>,

    pub reports: Reports,

    pub recording: Option<Recording>,

    pub internal: Internal,
}

impl Settings {
    /// Creates settings for testing
    ///
    /// Do not use in production
    pub fn test_settings(api_token: String) -> Settings {
        let port = 11333;
        let address = "localhost".into();
        let public_url = Url::parse(&format!("http://{address}:{port}")).unwrap();
        let controller = ControllerConfig {
            url: Url::parse("http://localhost:8000").unwrap(),
            api_key: ApiKey::new("controller", "secret"),
        };

        Settings {
            http: Http {
                address,
                port,
                api_keys: ApiKeys::new(vec![ApiKey::new("roomserver", api_token)]),
                enable_openapi: true,
                service_url: None,
                public_url,
            },
            controller: Some(controller),
            orchestrator: None,
            monitoring: None,
            metrics: None,
            tracing: None,
            conference: Conference {
                signaling_salt: SignalingSalt("abcdefghijklmnopqrstuvwx".into()),
                room_idle_timeout: conference::DEFAULT_IDLE_TIMEOUT,
            },
            defaults: None,
            reports: Default::default(),
            recording: None,
            internal: Default::default(),
        }
    }
}

impl TryFrom<SettingsFile> for Settings {
    type Error = anyhow::Error;

    fn try_from(value: SettingsFile) -> Result<Self, Self::Error> {
        Ok(Settings {
            http: value.http.into(),
            controller: value.controller,
            orchestrator: value.orchestrator,
            monitoring: value.monitoring.map(Into::into),
            metrics: value.metrics.map(Into::into),
            tracing: value.tracing.map(Into::into),
            conference: value.conference.into(),
            defaults: value.defaults.map(Into::into),
            reports: value.reports.unwrap_or_default().into(),
            recording: value.recording.map(Into::into),
            internal: value.internal.map(Into::into).unwrap_or_default(),
        })
    }
}
