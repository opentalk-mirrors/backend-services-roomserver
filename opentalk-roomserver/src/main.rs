// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>

//! This crate builds an executable that runs the RoomServer. It implements the
//! [_OpenTalk RoomServer Web API_](opentalk_roomserver_web_api).

use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, ToSocketAddrs as _},
    path::Path,
    result,
    sync::Arc,
    time::Duration,
};

use anyhow::Context;
use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;
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
mod tcp_multi_listener;

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
        let addresses = determine_socket_address(settings.http.address.as_deref(), metric.port)?;
        let (m_layer, metric_handle) = metrics::build_prometheus_layer();
        set.spawn(
            metrics::run_metric_server(
                addresses,
                metric.allowlist.clone(),
                metric_handle,
                app_state.subscribe(),
            )
            .map_err(decorate_error("Metric server exited with error")),
        );
        metric_layer = Some(m_layer);
    }

    let addresses = determine_socket_address(settings.http.address.as_deref(), settings.http.port)?;
    set.spawn(
        api::run_web_server(settings, addresses, app_state.clone(), metric_layer)
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

fn determine_socket_address(
    config_address: Option<&str>,
    config_port: u16,
) -> std::io::Result<Vec<SocketAddr>> {
    let to_socket_addrs = if let Some(addr) = config_address {
        Vec::from_iter((addr, config_port).to_socket_addrs()?)
    } else if is_ipv6_available() {
        Vec::from_iter((Ipv6Addr::UNSPECIFIED, config_port).to_socket_addrs()?)
    } else {
        Vec::from_iter((Ipv4Addr::UNSPECIFIED, config_port).to_socket_addrs()?)
    };
    Ok(to_socket_addrs)
}

fn is_ipv6_available() -> bool {
    TcpListener::bind((Ipv6Addr::UNSPECIFIED, 0)).is_ok()
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
    use std::{
        collections::BTreeSet,
        net::{Ipv4Addr, Ipv6Addr, SocketAddr},
        time::Duration,
    };

    use tokio::net::TcpStream;

    use crate::{determine_socket_address, is_ipv6_available};

    const PORT: u16 = 11333;

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

    #[test]
    fn localhost_multi_ip_when_v6_available() {
        let addresses = determine_socket_address(Some("localhost"), PORT).unwrap();

        if is_ipv6_available() {
            assert_eq!(
                BTreeSet::from_iter(addresses),
                BTreeSet::from_iter(vec![
                    SocketAddr::from((Ipv4Addr::LOCALHOST, PORT)),
                    SocketAddr::from((Ipv6Addr::LOCALHOST, PORT))
                ])
            );
        } else {
            assert_eq!(
                addresses,
                vec![SocketAddr::from((Ipv4Addr::LOCALHOST, PORT))]
            );
        }
    }

    #[test]
    fn single_ipv6() {
        let addresses =
            determine_socket_address(Some(&Ipv6Addr::LOCALHOST.to_string()), PORT).unwrap();
        assert_eq!(
            addresses,
            vec![SocketAddr::from((Ipv6Addr::LOCALHOST, PORT))]
        );
    }

    #[test]
    fn single_ipv4() {
        let addresses =
            determine_socket_address(Some(&Ipv4Addr::LOCALHOST.to_string()), PORT).unwrap();
        assert_eq!(
            addresses,
            vec![SocketAddr::from((Ipv4Addr::LOCALHOST, PORT))]
        );
    }
}
