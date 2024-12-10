// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::sync::Arc;

use anyhow::Result;
use axum::async_trait;
use axum_prometheus::{
    metrics_exporter_prometheus::PrometheusHandle, PrometheusMetricLayerBuilder,
};
use opentalk_roomserver_types::{room_parameters, room_parameters::RoomParameters};
use opentalk_roomserver_web_api::v1::{self, Backend, MetricBackend, RoomAction, RoomBackend};
use opentalk_types_common::rooms::RoomId;
use service_probe::{set_service_state, ServiceState};
use tokio::sync::watch;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{room::registry::RoomTaskRegistry, settings::Settings};

pub(crate) type Router = axum::Router<Context>;

pub mod signaling;

#[derive(OpenApi)]
#[openapi(
        info(
            title = "OpenTalk RoomServer API",
            description = "Specifies the endpoints and structure of the OpenTalk RoomServer Web API",
        ),
        tags(
            (name = "v1::rooms", description = "Endpoints related to rooms"),
            (name = "v1::metrics", description = "Endpoints related to metrics")
        ),
        paths(
           v1::rooms::put_room,
           v1::metrics::metrics,
        ),
        components(
            schemas(
                opentalk_types_common::call_in::CallInId,
                opentalk_types_common::call_in::CallInInfo,
                opentalk_types_common::call_in::CallInPassword,
                opentalk_types_common::call_in::NumericId,
                opentalk_types_common::features::FeatureId,
                opentalk_types_common::features::ModuleFeatureId,
                opentalk_types_common::modules::ModuleId,
                opentalk_types_common::rooms::RoomId,
                opentalk_types_common::shared_folders::SharedFolder,
                opentalk_types_common::shared_folders::SharedFolderAccess,
                opentalk_types_common::streaming::StreamingLink,
                opentalk_types_common::tariffs::TariffId,
                opentalk_types_common::tariffs::TariffModuleResource,
                opentalk_types_common::tariffs::TariffResource,
                opentalk_types_common::users::UserId,
                opentalk_types::api::v1::users::PublicUserProfile,
                room_parameters::EventInfo,
                room_parameters::RoomParameters,
            )
        )
    )]
pub(crate) struct ApiDoc;

#[derive(Debug, Clone, Copy, Default)]
pub enum ApplicationState {
    #[default]
    Running,

    _ShuttingDown,
}

impl ApplicationState {
    /// Returns `true` if the application state is [`ShuttingDown`].
    ///
    /// [`ShuttingDown`]: ApplicationState::_ShuttingDown
    pub fn is_shutting_down(&self) -> bool {
        matches!(self, Self::_ShuttingDown)
    }
}

/// Context for the API endpoints
#[derive(Clone)]
pub(crate) struct Context {
    settings: Arc<Settings>,
    /// Global list of room tasks and their handles
    room_tasks: RoomTaskRegistry,
    metric_handle: PrometheusHandle,
    app_state: watch::Sender<ApplicationState>,
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

    let (app_state, _) = watch::channel(ApplicationState::default());

    let ctx = Context {
        settings: settings.clone(),
        room_tasks: RoomTaskRegistry::default(),
        metric_handle,
        app_state,
    };

    let mut router = Router::new()
        .nest("/v1", v1::routes())
        .merge(v1::routes())
        .layer(metric_layer)
        .with_state(ctx);

    if !settings.http.disable_openapi {
        let mut openapi = ApiDoc::openapi();
        openapi.servers = Some(vec![utoipa::openapi::Server::new("/v1")]);
        router = router.merge(SwaggerUi::new("/swagger").url("/docs/openapi.json", openapi));
    }

    let listener =
        tokio::net::TcpListener::bind((settings.http.address, settings.http.port)).await?;

    log::info!("Listening on http://{}", listener.local_addr()?);

    set_service_state(ServiceState::Ready);
    axum::serve(listener, router).await?;

    Ok(())
}

impl Backend for Context {}

#[async_trait]
impl MetricBackend for Context {
    async fn render(&mut self) -> String {
        self.metric_handle.render()
    }
}

#[async_trait]
impl RoomBackend for Context {
    async fn put_room(
        &self,
        room_parameters: RoomParameters,
        room_id: RoomId,
    ) -> Result<RoomAction, opentalk_types::api::error::ApiError> {
        let (action, task_handle) = self
            .room_tasks
            .put_room(room_id, room_parameters, self.app_state.subscribe())
            .await
            .map_err(|err| {
                log::info!("Failed to put room {}: {err}", room_id);
                err
            })?;

        if !action.is_created() {
            // Refresh the idle timeout if the room was not created with this request
            task_handle.refresh_idle_timeout().await.map_err(|err| {
                log::info!("Failed to refresh idle timeout for room {}: {err}", room_id);
                err
            })?;
        }

        Ok(action)
    }
}
