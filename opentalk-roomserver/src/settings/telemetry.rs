// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::net::IpAddr;

use serde::Deserialize;

#[derive(Default, Debug, Clone, Deserialize)]
pub(crate) struct Metrics {
    #[serde(default = "default_metrics_port")]
    pub(crate) port: u16,
}

const fn default_metrics_port() -> u16 {
    11412
}

/// Configuration for the ready, startup, liveness probe.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Monitoring {
    /// Port on which the probe can be reached.
    #[serde(default = "default_monitor_port")]
    pub(crate) port: u16,

    /// Address which is used to listen for new connections.
    #[serde(default = "crate::settings::default_bind_address")]
    pub(crate) addr: IpAddr,
}

const fn default_monitor_port() -> u16 {
    11411
}

/// Configure a logging target.
#[derive(Default, Debug, Clone, Deserialize)]
pub(crate) struct Tracing {
    default_directives: Option<Vec<String>>,

    pub(crate) otlp_tracing_endpoint: String,

    pub(crate) service_name: Option<String>,

    pub(crate) service_namespace: Option<String>,

    pub(crate) service_instance_id: Option<String>,
}

impl Tracing {
    pub(crate) fn log_filter(&self) -> Option<String> {
        self.default_directives
            .as_ref()
            .map(|filter| filter.join(","))
    }
}
