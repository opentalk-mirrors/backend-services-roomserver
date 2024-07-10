// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::sync::Arc;

use crate::settings::Settings;
use anyhow::Result;
use axum_prometheus::{
    metrics_exporter_prometheus::PrometheusHandle, PrometheusMetricLayerBuilder,
};

mod v1;

pub(crate) type Router = axum::Router<Context>;

/// Context for the API endpoints
#[derive(Clone)]
pub(crate) struct Context {
    _settings: Arc<Settings>,
    metric_handle: PrometheusHandle,
}

/// Starts the web server
///
/// The api will be served under the `/v1/...` path. The version segment (`v1`) is optional. If the version is not
/// specified the latest api version is used.
pub(crate) async fn run_web_server(settings: Arc<Settings>) -> Result<()> {
    let (metric_layer, metric_handle) = PrometheusMetricLayerBuilder::new()
        .with_prefix("api")
        .enable_response_body_size(true)
        .with_default_metrics()
        .build_pair();

    let ctx = Context {
        _settings: settings.clone(),
        metric_handle,
    };

    let router = Router::new()
        .nest("/v1", v1::routes())
        .merge(v1::routes())
        .layer(metric_layer)
        .with_state(ctx);

    let listener =
        tokio::net::TcpListener::bind((settings.http.address.as_str(), settings.http.port)).await?;

    // TODO: add real logging
    println!("Listening on http://{}", listener.local_addr()?);

    axum::serve(listener, router).await?;

    Ok(())
}
