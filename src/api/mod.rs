// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::sync::Arc;

use anyhow::Result;
use axum::async_trait;
use axum_prometheus::{
    metrics::counter, metrics_exporter_prometheus::PrometheusHandle, PrometheusMetricLayerBuilder,
};
use opentalk_types::api::error::ApiError;
use opentalk_web_api::v1::{self, MetricHandle, RoomContext, RoomServerApi};

use crate::{room::registry::RoomTaskRegistry, settings::Settings};

pub(crate) type Router = axum::Router<Context>;

/// Context for the API endpoints
#[derive(Clone)]
pub(crate) struct Context {
    settings: Arc<Settings>,
    /// Global list of room tasks and their handles
    room_tasks: RoomTaskRegistry,
    metric_handle: PrometheusHandle,
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("settings", &self.settings)
            .field("room_tasks", &self.room_tasks)
            .finish()
    }
}

/// Starts the web server
///
/// The api will be served under the `/v1/...` path. The version segment (`v1`) is optional, if no version is specified
/// the latest api version is used.
pub(crate) async fn run_web_server(settings: Arc<Settings>) -> Result<()> {
    let (metric_layer, metric_handle) = PrometheusMetricLayerBuilder::new()
        .with_prefix("api")
        .enable_response_body_size(true)
        .with_default_metrics()
        .build_pair();

    let ctx = Context {
        settings: settings.clone(),
        room_tasks: RoomTaskRegistry::default(),
        metric_handle,
    };

    let router = Router::new()
        .nest("/v1", v1::routes())
        .merge(v1::routes())
        .layer(metric_layer)
        .with_state(ctx);

    let listener =
        tokio::net::TcpListener::bind((settings.http.address.as_str(), settings.http.port)).await?;

    log::info!("Listening on http://{}", listener.local_addr()?);

    axum::serve(listener, router).await?;

    Ok(())
}

impl RoomServerApi for Context {}

#[async_trait]
impl MetricHandle for Context {
    async fn render(&mut self) -> String {
        self.metric_handle.render()
    }
}

#[async_trait]
impl RoomContext for Context {
    async fn create_room_if_not_exists(
        &self,
        room_parameters: opentalk_web_api::types::RoomParameters,
    ) -> std::result::Result<(), opentalk_types::api::error::ApiError> {
        let room_id = room_parameters.room_id;

        let (created, task_handle) = self.room_tasks.create_room_if_not_exists(room_parameters);

        if created {
            return Ok(());
        }

        // Refresh the idle timeout if the room was not created with this request
        if let Err(err) = task_handle.refresh_idle_timeout().await {
            log::error!("Failed to refresh idle timeout for room {}: {err}", room_id);
            return Err(ApiError::internal());
        }

        Ok(())
    }

    async fn probe_room(&self, path: axum::extract::Path<String>) -> String {
        let room_id = path.0;

        log::trace!("Probing room {}", room_id);

        // Just an example for a custom metric (a counter in this case)
        counter!("probe_room_count_per_room", "room_id" => room_id.clone()).increment(1);

        format!("probing the room with id {}", room_id)
    }
}
