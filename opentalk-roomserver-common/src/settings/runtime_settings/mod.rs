// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use conference::Conference;
use defaults::Defaults;
use http::Http;
use opentalk_service_auth::{ApiKey, service::ApiKeys};
use reports::Reports;
use telemetry::{Metrics, Monitoring, Tracing};
use url::Url;

use super::{settings_file::SettingsFile, signaling_salt::SignalingSalt};

pub mod conference;
pub mod defaults;
pub mod http;
pub mod reports;
pub mod reports_typst;
pub mod telemetry;

#[derive(Debug, Clone)]
pub struct Settings {
    /// HTTP web server settings
    pub http: Http,

    pub monitoring: Option<Monitoring>,

    pub metrics: Option<Metrics>,

    pub tracing: Option<Tracing>,

    pub conference: Conference,

    pub defaults: Option<Defaults>,

    pub reports: Reports,
}

impl Settings {
    /// Creates settings for testing
    ///
    /// Do not use in production
    pub fn test_settings(api_token: String) -> Settings {
        let port = 11333;
        let address = "localhost".into();
        let public_url = Url::parse(&format!("http://{address}:{port}")).unwrap();

        Settings {
            http: Http {
                address,
                port,
                api_keys: ApiKeys::new(vec![ApiKey::new("roomserver", api_token)]),
                enable_openapi: true,
                public_url,
            },
            monitoring: None,
            metrics: None,
            tracing: None,
            conference: Conference {
                signaling_salt: SignalingSalt("abcdefghijklmnopqrstuvwx".into()),
                room_idle_timeout: conference::DEFAULT_IDLE_TIMEOUT,
            },
            defaults: None,
            reports: Default::default(),
        }
    }
}

impl From<SettingsFile> for Settings {
    fn from(value: SettingsFile) -> Self {
        Settings {
            http: value.http.into(),
            monitoring: value.monitoring.map(Into::into),
            metrics: value.metrics.map(Into::into),
            tracing: value.tracing.map(Into::into),
            conference: value.conference.into(),
            defaults: value.defaults.map(Into::into),
            reports: value.reports.unwrap_or_default().into(),
        }
    }
}
