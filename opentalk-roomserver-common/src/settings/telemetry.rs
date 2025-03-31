// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::net::IpAddr;

use serde::Deserialize;

#[derive(Default, Debug, Clone, Deserialize)]
pub struct Metrics {
    #[serde(default = "default_metrics_port")]
    pub port: u16,
}

const fn default_metrics_port() -> u16 {
    11412
}

/// Configuration for the ready, startup, liveness probe.
#[derive(Debug, Clone, Deserialize)]
pub struct Monitoring {
    /// Port on which the probe can be reached.
    #[serde(default = "default_monitor_port")]
    pub port: u16,

    /// Address which is used to listen for new connections.
    #[serde(default = "crate::settings::default_bind_address")]
    pub addr: IpAddr,
}

const fn default_monitor_port() -> u16 {
    11411
}

/// Configure a logging target.
#[derive(Default, Debug, Clone, Deserialize)]
pub struct Tracing {
    default_directives: Option<Vec<String>>,

    pub otlp_tracing_endpoint: String,

    pub service_name: Option<String>,

    pub service_namespace: Option<String>,

    pub service_instance_id: Option<String>,
}

impl Tracing {
    pub fn log_filter(&self) -> Option<String> {
        self.default_directives
            .as_ref()
            .map(|filter| filter.join(","))
    }
}
