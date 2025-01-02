// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::net::IpAddr;

use anyhow::Context as _;
use axum::{extract::State, routing::get};
use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;
use tokio::sync::watch;

use crate::{wait_shutdown, ApplicationState};

pub(crate) type MetricRouter = axum::Router<MetricContext>;

pub(crate) async fn run_metric_server(
    address: IpAddr,
    port: u16,
    metric_handle: PrometheusHandle,
    app_state: watch::Receiver<ApplicationState>,
) -> Result<(), anyhow::Error> {
    let ctx = MetricContext { metric_handle };

    let router = MetricRouter::new()
        .route("/metrics", get(metrics))
        .with_state(ctx);

    let listener = tokio::net::TcpListener::bind((address, port))
        .await
        .context(format!("Failed to bind metrics to port {port}"))?;
    log::info!(
        "Listening for metrics on http://{}",
        listener.local_addr().expect("Failed to get local address")
    );
    axum::serve(listener, router)
        .with_graceful_shutdown(wait_shutdown(app_state))
        .await
        .context("Failed to serve metrics")
}

#[derive(Clone)]
pub(crate) struct MetricContext {
    pub metric_handle: PrometheusHandle,
}

impl MetricContext {
    async fn render(&mut self) -> String {
        self.metric_handle.render()
    }
}

/// Returns the prometheus metrics
///
/// Returns the prometheus metrics in a text format.
#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = StatusCode::OK, description = "Prometheus metrics were successfully retrieved"),
    ),
    security(),
    )]
async fn metrics(mut context: State<MetricContext>) -> String {
    context.render().await
}
