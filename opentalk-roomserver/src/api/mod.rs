// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use axum::{
    extract::{ws::WebSocket, MatchedPath},
    http::Request,
};
use opentalk_roomserver_types::{
    client_parameters::ClientParameters, room_parameters, room_parameters::RoomParameters,
    signaling_context::SignalingClientContext,
};
use opentalk_roomserver_web_api::v1::{self, Backend, RoomAction, RoomBackend};
use opentalk_types_api_v1::error::ApiError;
use opentalk_types_common::{rooms::RoomId, roomserver::Token};
use service_probe::{set_service_state, ServiceState};
use token_store::TokenStore;
use tokio::sync::{watch, Mutex};
use tower_http::trace::TraceLayer;
use tracing::info_span;
use utoipa::{
    openapi::security::{Http, HttpAuthScheme},
    OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    room::{registry::RoomTaskRegistry, signaling::module_initializer::ModuleRegistry},
    settings::Settings,
    wait_shutdown, ApplicationState,
};

pub(crate) type Router = axum::Router<Context>;

pub mod signaling;
mod token_store;

#[derive(OpenApi)]
#[openapi(
        info(
            title = "OpenTalk RoomServer API",
            description = "Specifies the endpoints and structure of the OpenTalk RoomServer Web API",
        ),
        tags(
            (name = "v1::rooms", description = "Endpoints related to rooms"),
        ),
        paths(
           v1::rooms::put_room,
           v1::rooms::request_token
        ),
        components(
            schemas(
                opentalk_types_api_v1::users::PublicUserProfile,
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
                opentalk_types_common::users::DisplayName,
                opentalk_types_common::users::UserId,
                opentalk_types_common::users::UserTitle,
                room_parameters::EventContext,
                room_parameters::RoomParameters,
            )
        ),
        modifiers(&SecurityAddon),
    )]
pub(crate) struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::SecurityScheme;

        let components = openapi.components.as_mut().unwrap();

        let http_scheme = Http::builder()
            .scheme(HttpAuthScheme::Bearer)
            .bearer_format("api token")
            .description("The roomservers API token is expected to be in the `Authorization` header with the format: `bearer <token>`".into())
            .build();

        components.add_security_scheme("API-Token", SecurityScheme::Http(http_scheme));
    }
}

/// Starts the web server
///
/// The api will be served under the `/v1/...` path. The version segment (`v1`) is optional, if no version is specified
/// the latest api version is used.
pub(crate) async fn run_web_server<L>(
    settings: Arc<Settings>,
    app_state: watch::Sender<ApplicationState>,
    metric_layer: Option<L>,
) -> anyhow::Result<()>
where
    L: tower::Layer<axum::routing::Route> + Clone + Send + Sync + 'static,
    L::Service: tower::Service<axum::extract::Request> + Clone + Send + Sync + 'static,
    <L::Service as tower::Service<axum::extract::Request>>::Response:
        axum::response::IntoResponse + 'static,
    <L::Service as tower::Service<axum::extract::Request>>::Error:
        Into<std::convert::Infallible> + 'static,
    <L::Service as tower::Service<axum::extract::Request>>::Future: Send + 'static,
{
    let app_state_subscriber = app_state.subscribe();
    let ctx = Context {
        settings: Arc::clone(&settings),
        room_tasks: RoomTaskRegistry::new(),
        token_store: Arc::new(Mutex::new(TokenStore::new())),
        module_registry: Arc::new(ModuleRegistry::new()),
        app_state,
    };

    let mut router = Router::new()
        .nest("/v1", v1::routes(settings.http.api_token.clone()))
        .merge(v1::routes(settings.http.api_token.clone()))
        .with_state(ctx)
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let matched_path = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str);

                info_span!(
                    "http_request",
                    http.request.method = ?request.method(),
                    http.route = matched_path,
                )
            }),
        );

    if let Some(layer) = metric_layer {
        router = router.layer(layer);
    }

    if !settings.http.disable_openapi {
        // TODO: Having this enabled causes the utoipa schema to be cloned and dropped for each request which increases cost by about ~40%
        let mut openapi = ApiDoc::openapi();
        openapi.servers = Some(vec![utoipa::openapi::Server::new("/v1")]);
        router = router.merge(SwaggerUi::new("/swagger").url("/docs/openapi.json", openapi));
    }

    let listener =
        tokio::net::TcpListener::bind((settings.http.address, settings.http.port)).await?;

    log::info!("Listening on http://{}", listener.local_addr()?);

    set_service_state(ServiceState::Ready);
    axum::serve(listener, router)
        .with_graceful_shutdown(wait_shutdown(app_state_subscriber))
        .await?;

    Ok(())
}

/// Context for the API endpoints
#[derive(Clone)]
pub(crate) struct Context {
    settings: Arc<Settings>,
    /// Global list of room tasks and their handles
    room_tasks: RoomTaskRegistry<WebSocket>,
    // A list of eligible participants and their join tokens
    token_store: Arc<Mutex<TokenStore>>,
    module_registry: Arc<ModuleRegistry>,

    app_state: watch::Sender<ApplicationState>,
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("settings", &self.settings)
            .field("room_tasks", &self.room_tasks)
            .finish_non_exhaustive()
    }
}

impl Context {
    /// Spawn the room task from the given room parameters
    ///
    /// If the room is already running, the rooms idle timeout is refreshed and the given RoomParameters will be ignored
    async fn prepare_room(
        &self,
        room_id: RoomId,
        room_parameters: RoomParameters,
    ) -> Result<(), ApiError> {
        let Some(room_handle) = self
            .room_tasks
            .create_or_get(
                room_id,
                room_parameters,
                Arc::clone(&self.module_registry),
                Arc::clone(&self.settings),
                self.app_state.subscribe(),
            )
            .await
        else {
            // room has been created
            return Ok(());
        };

        // refresh the idle timeout of the existing room to avoid race conditions
        if let Err(e) = room_handle.refresh_idle_timeout().await {
            // This can only fail if the rooms idle timeout has been reached or the room has been manually removed
            log::error!("Failed to refresh idle timeout of room {room_id}: {e}");

            return Err(ApiError::internal());
        }

        Ok(())
    }
}

impl Backend for Context {}

#[async_trait]
impl RoomBackend for Context {
    async fn put_room(
        &self,
        room_id: RoomId,
        room_parameters: RoomParameters,
    ) -> Result<RoomAction, opentalk_types_api_v1::error::ApiError> {
        let (action, task_handle) = self
            .room_tasks
            .put_room(
                room_id,
                room_parameters,
                Arc::clone(&self.module_registry),
                Arc::clone(&self.settings),
                self.app_state.subscribe(),
            )
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

    async fn request_room_token(
        &mut self,
        room_id: RoomId,
        client_parameters: ClientParameters,
        room_parameters: Option<RoomParameters>,
    ) -> Result<Option<Token>, opentalk_types_api_v1::error::ApiError> {
        match room_parameters {
            Some(parameters) => self.prepare_room(room_id, parameters).await?,
            None => {
                let Some(task_handle) = self.room_tasks.get_task_handle(&room_id).await else {
                    return Ok(None);
                };

                task_handle.refresh_idle_timeout().await?;
            }
        }

        let token = self
            .token_store
            .lock()
            .await
            .create_token(SignalingClientContext::new(room_id, client_parameters));

        Ok(Some(token))
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use opentalk_roomserver_types::client_parameters::ClientKind;
    use opentalk_types_api_v1::users::PublicUserProfile;
    use opentalk_types_common::{
        tariffs::TariffResource,
        users::{DisplayName, UserId, UserInfo, UserTitle},
        utils::ExampleData,
    };

    use super::*;

    fn test_context() -> Context {
        let settings: Arc<Settings> = Arc::new(Settings::test_settings("secret".into()));
        let (app_state, _) = watch::channel(ApplicationState::Running);

        Context {
            settings: settings.clone(),
            room_tasks: RoomTaskRegistry::new(),
            token_store: Arc::new(Mutex::new(TokenStore::new())),
            module_registry: Arc::new(ModuleRegistry::new()),
            app_state,
        }
    }

    fn client_parameters2() -> ClientParameters {
        ClientParameters {
            client_id: "1234".into(),
            kind: ClientKind::Guest {
                display_name: DisplayName::from_str_lossy("tester"),
            },
        }
    }

    fn client_parameters1() -> ClientParameters {
        ClientParameters {
            client_id: "1234".into(),
            kind: ClientKind::Registered {
                profile: PublicUserProfile {
                    id: UserId::nil(),
                    email: "example@opentalk.eu".into(),
                    user_info: UserInfo {
                        title: UserTitle::new(),
                        firstname: "Test".into(),
                        lastname: "Tester".into(),
                        display_name: DisplayName::from_str_lossy("tester"),
                        avatar_url: "example.com".into(),
                    },
                },
            },
        }
    }

    fn room_parameters() -> RoomParameters {
        RoomParameters {
            created_by: PublicUserProfile::example_data(),
            password: None,
            waiting_room: false,
            call_in: None,
            event: None,
            invite_code: None,
            tariff: TariffResource::example_data(),
            streaming_links: vec![],
            e2e_encryption: false,
        }
    }

    #[tokio::test]
    async fn put_room() {
        let ctx = test_context();

        let id = RoomId::from_u128(0xf4bc4806_a35c_4ce0_bcb3_fb990b287d4c);
        let action = ctx.put_room(id, room_parameters()).await.unwrap();
        assert_eq!(action, RoomAction::Created);

        // TODO add second put_room request and check for UPDATED response once implemented
    }

    #[tokio::test]
    async fn request_token() {
        let mut ctx = test_context();

        let token = ctx
            .request_room_token(RoomId::nil(), client_parameters1(), None)
            .await
            .unwrap();

        assert_eq!(token, None);

        let token = ctx
            .request_room_token(RoomId::nil(), client_parameters1(), Some(room_parameters()))
            .await
            .unwrap();

        assert!(token.is_some());

        let token = ctx
            .request_room_token(RoomId::nil(), client_parameters2(), None)
            .await
            .unwrap();

        assert!(token.is_some())
    }
}
