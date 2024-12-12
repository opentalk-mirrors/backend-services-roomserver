// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>

//! This crate builds an executable that runs the RoomServer. It implements the [_OpenTalk RoomServer Web API_][opentalk_roomserver_web_api].

use std::sync::Arc;

use anyhow::Context;
use axum_prometheus::{
    metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle},
    utils::SECONDS_DURATION_BUCKETS,
    GenericMetricLayer, PrometheusMetricLayerBuilder, AXUM_HTTP_REQUESTS_DURATION_SECONDS,
};
use clap::Parser;
use cli::{Args, SubCommand};
use service_probe::{start_probe, ServiceState};
use settings::Settings;

mod api;
mod cli;
mod metrics;
#[cfg(test)]
mod mocking;
mod room;
pub(crate) mod settings;
mod trace;

async fn run_web_server(config_file_name: &str) -> anyhow::Result<()> {
    let settings = Arc::new(Settings::load(config_file_name)?);

    trace::init(settings.tracing.as_ref()).context("Failed to initialize tracing")?;
    if let Some(monitoring) = &settings.monitoring {
        start_probe(monitoring.addr, monitoring.port, ServiceState::Up)
            .await
            .context("Failed to start monitoring endpoint")?;
    }
    // TODO handle metrics server errors
    let (metric_layer, metric_handle) = build_prometheus_layer();
    tokio::spawn(api::run_metric_server(
        settings.http.address,
        settings.metrics.port,
        metric_handle,
    ));
    api::run_web_server(settings, metric_layer).await?;

    Ok(())
}

fn build_prometheus_layer<'a>() -> (
    GenericMetricLayer<'a, PrometheusHandle, axum_prometheus::Handle>,
    PrometheusHandle,
) {
    PrometheusMetricLayerBuilder::new()
        .with_prefix("api")
        .enable_response_body_size(true)
        // Using with_metrics_from instead of with_default_metrics because
        // with_default_metrics crashes when port 9000 is already in use,
        // see https://github.com/Ptrskay3/axum-prometheus/issues/66
        .with_metrics_from_fn(|| {
            PrometheusBuilder::new()
                .set_buckets_for_metric(
                    Matcher::Full(AXUM_HTTP_REQUESTS_DURATION_SECONDS.to_string()),
                    SECONDS_DURATION_BUCKETS,
                )
                .expect("Setting prometheus buckets failed")
                .install_recorder()
                .expect("Installing prometheus recorder failed")
        })
        .build_pair()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.run_tasks().should_exit() {
        return Ok(());
    }

    match args.cmd {
        Some(SubCommand::Openapi(command)) => {
            cli::openapi::handle_command(command).await?;
        }
        None => run_web_server(&args.config).await?,
    }

    Ok(())
}
