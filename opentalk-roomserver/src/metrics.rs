// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::net::SocketAddr;

use anyhow::Context as _;
use axum::{
    Router,
    extract::{ConnectInfo, State},
    http::StatusCode,
    routing::get,
    serve::Listener as _,
};
use axum_prometheus::{
    AXUM_HTTP_REQUESTS_DURATION_SECONDS, GenericMetricLayer, PrometheusMetricLayerBuilder,
    metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle},
    utils::SECONDS_DURATION_BUCKETS,
};
use cidr::IpInet;
use opentalk_roomserver_room::metrics::{
    CONNECTION_MEETING_TIME, CONNECTION_MEETING_TIME_BUCKETS, ROOM_LIFE_TIME,
    ROOM_LIFE_TIME_BUCKETS,
};
use tokio::sync::watch;

use crate::{
    ApplicationState,
    tcp_multi_listener::{MultiAddr, MultiListener},
    wait_shutdown,
};

pub(super) fn build_prometheus_layer<'a>() -> (
    GenericMetricLayer<'a, PrometheusHandle, axum_prometheus::Handle>,
    PrometheusHandle,
) {
    PrometheusMetricLayerBuilder::new()
        .with_prefix("api")
        .enable_response_body_size(true)
        .with_metrics_from_fn(|| {
            PrometheusBuilder::new()
                .set_buckets_for_metric(
                    Matcher::Full(AXUM_HTTP_REQUESTS_DURATION_SECONDS.to_string()),
                    SECONDS_DURATION_BUCKETS,
                )
                .expect("Setting prometheus buckets failed")
                .set_buckets_for_metric(
                    Matcher::Full(CONNECTION_MEETING_TIME.to_string()),
                    CONNECTION_MEETING_TIME_BUCKETS,
                )
                .expect("Setting prometheus meeting time buckets failed")
                .set_buckets_for_metric(
                    Matcher::Full(ROOM_LIFE_TIME.to_string()),
                    ROOM_LIFE_TIME_BUCKETS,
                )
                .expect("Setting prometheus room life time buckets failed")
                .install_recorder()
                .expect("Installing prometheus recorder failed")
        })
        .build_pair()
}

pub(crate) async fn run_metric_server<H>(
    addresses: Vec<SocketAddr>,
    allowlist: Vec<IpInet>,
    metric_handle: H,
    app_state: watch::Receiver<ApplicationState>,
) -> Result<(), anyhow::Error>
where
    H: MetricHandle + Clone + Send + Sync + 'static,
{
    let ctx = MetricContext {
        metric_handle,
        allowlist,
    };

    let router = Router::<MetricContext<H>>::new()
        .route("/metrics", get(metrics))
        .with_state(ctx);
    let listener = MultiListener::bind(addresses.clone())
        .await
        .with_context(|| {
            format!(
                "Failed to bind metrics to: {}",
                addresses
                    .iter()
                    .map(|addr| format!("{addr}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;
    tracing::info!(
        "Listening for metrics on {}",
        listener.local_addr().expect("Failed to get local address")
    );

    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<MultiAddr>(),
    )
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
    pub allowlist: Vec<IpInet>,
}

impl<H: MetricHandle> MetricContext<H> {
    fn render(&mut self) -> String {
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
        (status = StatusCode::FORBIDDEN, description = "The client IP is not allowed to access the metrics endpoint"),
    ),
    security(),
    )]
async fn metrics<H: MetricHandle>(
    mut context: State<MetricContext<H>>,
    ConnectInfo(addr): ConnectInfo<MultiAddr>,
) -> Result<String, StatusCode> {
    let addr = match addr.addrs.as_slice() {
        [a] => a,
        _ => {
            tracing::error!(
                "Expected a single address in ConnectInfo, got multiple or none: {:?}",
                addr.addrs
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let ip = addr.ip();
    if context.allowlist.iter().any(|net| net.contains(&ip)) {
        return Ok(context.render());
    }

    if context.allowlist.is_empty() {
        tracing::debug!(
            "An attempt to access the metrics endpoint from IP address {ip} was denied. Access to the metrics endpoint has not been configured."
        );
    } else {
        let allowed_nets = context
            .allowlist
            .iter()
            .map(|net| format!("\"{net}\""))
            .collect::<Vec<String>>()
            .join(", ");
        tracing::debug!(
            "An attempt to access the metrics endpoint from IP address {ip} was denied. Access allowed from: {allowed_nets}."
        );
    }

    Err(StatusCode::FORBIDDEN)
}

#[cfg(test)]
mod tests {
    use std::{
        net::{IpAddr, Ipv4Addr, SocketAddr},
        vec,
    };

    use opentalk_roomserver_common::application_state::ApplicationState;
    use reqwest::StatusCode;
    use tokio::sync::watch;

    use super::run_metric_server;
    use crate::{metrics::MetricHandle, tests::wait_for_server};

    const LOCALHOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    const RESPONSE: &str = "metrics";

    #[derive(Clone, Copy)]
    struct MockMetricHandle;

    impl MetricHandle for MockMetricHandle {
        fn render(&self) -> String {
            RESPONSE.to_string()
        }
    }

    #[test_log::test(tokio::test)]
    async fn allowlist_not_configured() {
        // Using a different port for each test to avoid conflicts when run in parallel
        const PORT: u16 = 11410;

        // Start the metric server without any allowed IPs
        let (app_state, _) = watch::channel(ApplicationState::Running);
        let addresses = vec![SocketAddr::new(LOCALHOST, PORT)];
        tokio::spawn(run_metric_server(
            addresses,
            vec![],
            MockMetricHandle,
            app_state.subscribe(),
        ));

        wait_for_server(SocketAddr::new(LOCALHOST, PORT)).await;

        let status = reqwest::get(format!("http://{LOCALHOST}:{PORT}/metrics"))
            .await
            .unwrap()
            .status();
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[test_log::test(tokio::test)]
    async fn localhost_forbidden() {
        const PORT: u16 = 11411;

        // Start the metric server without any allowed IPs
        let (app_state, _) = watch::channel(ApplicationState::Running);
        let addresses = vec![SocketAddr::new(LOCALHOST, PORT)];
        tokio::spawn(run_metric_server(
            addresses,
            vec![Ipv4Addr::new(192, 168, 0, 1).into()],
            MockMetricHandle,
            app_state.subscribe(),
        ));

        wait_for_server(SocketAddr::new(LOCALHOST, PORT)).await;

        let status = reqwest::get(format!("http://{LOCALHOST}:{PORT}/metrics"))
            .await
            .unwrap()
            .status();
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[test_log::test(tokio::test)]
    async fn localhost_allowed() {
        const PORT: u16 = 11412;

        // Start the metric server allowing localhost
        let (app_state, _) = watch::channel(ApplicationState::Running);
        let addresses = vec![SocketAddr::new(LOCALHOST, PORT)];
        tokio::spawn(run_metric_server(
            addresses,
            vec![LOCALHOST.into()],
            MockMetricHandle,
            app_state.subscribe(),
        ));

        wait_for_server(SocketAddr::new(LOCALHOST, PORT)).await;

        let response = reqwest::get(format!("http://{LOCALHOST}:{PORT}/metrics"))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.text().await.unwrap(), RESPONSE);
    }
}
