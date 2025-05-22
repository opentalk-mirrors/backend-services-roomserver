// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::net::IpAddr;

use anyhow::Context as _;
use axum::{Router, extract::State, routing::get};
use tokio::sync::watch;

use crate::{ApplicationState, wait_shutdown};

pub(crate) async fn run_metric_server<H>(
    address: IpAddr,
    port: u16,
    metric_handle: H,
    app_state: watch::Receiver<ApplicationState>,
) -> Result<(), anyhow::Error>
where
    H: MetricHandle + Clone + Send + Sync + 'static,
{
    let ctx = MetricContext { metric_handle };

    let router = Router::<MetricContext<H>>::new()
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

pub(crate) trait MetricHandle {
    fn render(&self) -> String;
}

#[derive(Clone)]
struct MetricContext<H: MetricHandle> {
    pub metric_handle: H,
}

impl<H: MetricHandle> MetricContext<H> {
    async fn render(&mut self) -> String {
        self.metric_handle.render()
    }
}

/// Returns the metrics
///
/// Returns the metrics in a text format.
#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = StatusCode::OK, description = "Metrics were successfully retrieved"),
    ),
    security(),
    )]
async fn metrics<H: MetricHandle>(mut context: State<MetricContext<H>>) -> String {
    context.render().await
}
