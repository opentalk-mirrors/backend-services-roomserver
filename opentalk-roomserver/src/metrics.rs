// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use axum::{extract::State, routing::get};
use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;

pub(crate) type MetricRouter = axum::Router<MetricContext>;

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
pub(crate) async fn metrics(mut context: State<MetricContext>) -> String {
    context.render().await
}

pub fn routes() -> MetricRouter {
    MetricRouter::new().route("/metrics", get(metrics))
}
