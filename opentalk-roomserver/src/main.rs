// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>

//! This crate builds an executable that runs the RoomServer. It implements the
//! [_OpenTalk RoomServer Web API_](opentalk_roomserver_web_api).

use std::{path::Path, result, sync::Arc, time::Duration};

use anyhow::Context;
use axum_prometheus::{
    AXUM_HTTP_REQUESTS_DURATION_SECONDS, GenericMetricLayer, PrometheusMetricLayerBuilder,
    metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle},
    utils::SECONDS_DURATION_BUCKETS,
};
use clap::Parser;
use cli::{Args, SubCommand};
use futures::TryFutureExt;
use metrics::MetricHandle;
use opentalk_roomserver_common::{
    application_state::ApplicationState,
    settings::{Monitoring, Settings, SettingsFile},
};
use service_probe::{ServiceState, start_probe, stop_probe};
use tokio::{
    signal,
    sync::watch::{self, Receiver},
    task::JoinSet,
    time::{Instant, timeout_at},
};

mod api;
mod cli;
mod metrics;

mod trace;

const SHUTDOWN_GRACE_PERIOD: Duration = Duration::from_secs(42);

pub(crate) async fn wait_shutdown(mut app_state: watch::Receiver<ApplicationState>) {
    let res = app_state.wait_for(ApplicationState::is_shutting_down).await;
    if let Err(e) = res {
        tracing::error!("AppState gone: {e}");
    }
}

pub fn decorate_error(decoration: &'static str) -> impl Fn(anyhow::Error) -> anyhow::Error {
    move |err| err.context(decoration)
}

async fn run_app(config_file_path: Option<&Path>) -> anyhow::Result<()> {
    let (app_state, _) = watch::channel(ApplicationState::Running);
    let settings: Arc<Settings> = Arc::new(SettingsFile::load(config_file_path)?.into());
    let mut set = JoinSet::new();

    set.spawn(
        shutdown_signal(app_state.subscribe())
            .map_err(decorate_error("Shutdown handler exited with error")),
    );

    trace::init(settings.tracing.as_ref()).context("Failed to initialize tracing")?;

    if let Some(monitoring) = &settings.monitoring {
        set.spawn(
            start_service_probe(monitoring.clone(), app_state.subscribe())
                .map_err(decorate_error("Service prove exited with error")),
        );
    }

    let mut metric_layer = None;
    if let Some(metric) = &settings.metrics {
        let (m_layer, metric_handle) = build_prometheus_layer();
        set.spawn(
            metrics::run_metric_server(
                settings.http.address,
                metric.port,
                metric.allowlist.clone(),
                metric_handle,
                app_state.subscribe(),
            )
            .map_err(decorate_error("Metric server exited with error")),
        );
        metric_layer = Some(m_layer);
    }

    set.spawn(
        api::run_web_server(settings, app_state.clone(), metric_layer)
            .map_err(decorate_error("API server exited with error")),
    );

    match set.join_next().await {
        // No task was started, this should not happen
        None => tracing::error!("Failed to start any task!"),
        // Task panicked
        Some(Err(e)) => tracing::error!("Task panicked: {e:?}"),
        // Task finished successfully
        Some(Ok(Ok(()))) => {}
        // Task returned an error
        Some(Ok(Err(e))) => tracing::error!("{e:?}"),
    }
    let result = graceful_shutdown(app_state, &mut set).await;
    if result.is_err() {
        set.abort_all();
        return result.context("Forced Shutdown");
    }

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

impl MetricHandle for PrometheusHandle {
    fn render(&self) -> String {
        self.render()
    }
}

async fn graceful_shutdown(
    app_state: watch::Sender<ApplicationState>,
    set: &mut JoinSet<result::Result<(), anyhow::Error>>,
) -> anyhow::Result<()> {
    tracing::debug!("Starting graceful shutdown");
    app_state.send_replace(ApplicationState::ShuttingDown);
    loop {
        let result = timeout_at(Instant::now() + SHUTDOWN_GRACE_PERIOD, set.join_next()).await;
        match result {
            // Timeout elapsed
            Err(_) => {
                tracing::error!("Not all tasks exited in time!");
                return Err(anyhow::anyhow!("Not all tasks exited in time!"));
            }
            // All tasks shut down
            Ok(None) => return Ok(()),
            // Task exited successfully
            Ok(Some(Ok(Ok(())))) => tracing::info!("Task exited"),
            // Task returned error
            Ok(Some(Ok(Err(e)))) => tracing::error!("Task error: {e:?}"),
            // Task panicked
            Ok(Some(Err(e))) => tracing::error!("Task Panic: {e:?}"),
        }
    }
}

async fn shutdown_signal(app_state: Receiver<ApplicationState>) -> anyhow::Result<()> {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
        () = wait_shutdown(app_state) => {},
    }

    Ok(())
}

async fn start_service_probe(
    monitoring: Monitoring,
    mut app_state_receiver: Receiver<ApplicationState>,
) -> Result<(), anyhow::Error> {
    start_probe(monitoring.addr, monitoring.port, ServiceState::Up)
        .await
        .context("Failed to start monitoring endpoint")?;
    app_state_receiver
        .wait_for(ApplicationState::is_shutting_down)
        .await?;
    stop_probe().await;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.run_tasks().should_exit() {
        return Ok(());
    }

    match args.cmd {
        Some(SubCommand::Openapi(command)) => {
            cli::openapi::handle_command(command)?;
        }
        None => run_app(args.config.as_deref()).await?,
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{net::SocketAddr, time::Duration};

    use tokio::net::TcpStream;

    pub async fn wait_for_server(addr: SocketAddr) {
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if TcpStream::connect(addr).await.is_ok() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("metrics server did not become ready in time");
    }
}
