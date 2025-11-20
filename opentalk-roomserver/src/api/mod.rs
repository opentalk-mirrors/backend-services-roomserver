// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::sync::Arc;

use anyhow::{Context as _, Result};
use async_trait::async_trait;
use axum::{
    extract::MatchedPath,
    http::{Request, Response},
};
use opentalk_roomserver_common::settings::Settings;
use opentalk_roomserver_module_automod::AutomodModule;
use opentalk_roomserver_module_chat::ChatModule;
use opentalk_roomserver_module_e2ee::E2eeModule;
use opentalk_roomserver_module_echo::EchoModule;
use opentalk_roomserver_module_legal_vote::LegalVoteModule;
use opentalk_roomserver_module_livekit::LiveKitModule;
use opentalk_roomserver_module_meeting_notes::MeetingNotesModule;
use opentalk_roomserver_module_meeting_report::MeetingReportModule;
use opentalk_roomserver_module_moderation::ModerationModule;
use opentalk_roomserver_module_polls::PollsModule;
use opentalk_roomserver_module_raise_hands::RaiseHandsModule;
use opentalk_roomserver_module_shared_folder::SharedFolderModule;
use opentalk_roomserver_module_subroom_audio::SubroomAudioModule;
use opentalk_roomserver_module_timer::TimerModule;
use opentalk_roomserver_module_training_participation_report::TrainingParticipationReportModule;
use opentalk_roomserver_module_whiteboard::WhiteboardModule;
use opentalk_roomserver_room::{ModuleRegistry, RoomTaskRegistry};
use opentalk_roomserver_types::{
    api::RoomServerAccess,
    client_parameters::ClientParameters,
    room_parameters::{self, RoomParameters},
    signaling::signaling_context::SignalingClientContext,
};
use opentalk_roomserver_web_api::v1::{self, Backend, RoomAction, RoomBackend};
use opentalk_types_api_v1::error::ApiError;
use opentalk_types_common::rooms::RoomId;
use service_probe::{ServiceState, set_service_state};
use token_store::TokenStore;
use tokio::sync::{Mutex, watch};
use tower_http::trace::TraceLayer;
use tracing::{Span, info_span};
use utoipa::{
    OpenApi,
    openapi::security::{Http, HttpAuthScheme},
};
use utoipa_swagger_ui::SwaggerUi;

use crate::{ApplicationState, api::websocket::WebSocketAdapter, wait_shutdown};

pub(crate) type Router = axum::Router<Context>;

pub mod signaling;
mod token_store;
pub mod websocket;

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
           v1::rooms::request_token,
           v1::signaling::open_signaling_socket
        ),
        components(
            schemas(
                opentalk_types_api_v1::users::PublicUserProfile,
                opentalk_types_common::call_in::CallInId,
                opentalk_types_common::call_in::CallInInfo,
                opentalk_types_common::call_in::CallInPassword,
                opentalk_types_common::call_in::NumericId,
                opentalk_types_common::features::FeatureId,
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

    let module_registry = setup_registry();
    let room_registry = RoomTaskRegistry::new(settings.conference.room_idle_timeout);

    let ctx = Context {
        settings: Arc::clone(&settings),
        room_tasks: room_registry.clone(),
        token_store: Arc::new(Mutex::new(TokenStore::new())),
        module_registry: Arc::new(module_registry),
        app_state,
    };

    let auth_middleware = settings
        .http
        .api_keys
        .auth_middleware()
        .context("Invalid API key configuration")?;

    let mut router = Router::new()
        .nest("/v1", v1::routes(auth_middleware))
        .with_state(ctx)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);

                    info_span!(
                        "http_request",
                        http.request.method = ?request.method(),
                        http.route = matched_path,
                        http.status_code = tracing::field::Empty,
                    )
                })
                .on_response(|response: &Response<_>, _duration, span: &Span| {
                    span.record(
                        "http.status_code",
                        tracing::field::display(response.status()),
                    );
                    if response.status().is_client_error() {
                        tracing::debug!(
                            "Request failed with status code: {} - (client error)",
                            response.status()
                        );
                    } else if response.status().is_server_error() {
                        tracing::error!(
                            "Request failed with status code: {} - (server error)",
                            response.status()
                        );
                    }
                })
                .on_failure(|fail, _duration, _span: &Span| {
                    tracing::warn!("request failed: {fail:?}");
                }),
        );

    if let Some(layer) = metric_layer {
        router = router.layer(layer);
    }

    if !settings.http.disable_openapi {
        // TODO: Having this enabled causes the utoipa schema to be cloned and dropped for each
        // request which increases cost by about ~40%
        let mut openapi = ApiDoc::openapi();
        openapi.servers = Some(vec![utoipa::openapi::Server::new("/v1")]);
        router = router.merge(SwaggerUi::new("/swagger").url("/docs/openapi.json", openapi));
    }

    let listener =
        tokio::net::TcpListener::bind((settings.http.address, settings.http.port)).await?;

    tracing::info!("Listening on http://{}", listener.local_addr()?);

    set_service_state(ServiceState::Ready);
    axum::serve(listener, router)
        .with_graceful_shutdown(wait_shutdown(app_state_subscriber))
        .await?;

    // wait for room tasks to close
    room_registry.wait_for_room_closed().await;

    Ok(())
}

/// Initialize the registry with all modules that are available for meetings
fn setup_registry() -> ModuleRegistry {
    let mut module_registry = ModuleRegistry::new();
    module_registry.add_module::<AutomodModule>();
    module_registry.add_module::<ChatModule>();
    module_registry.add_module::<E2eeModule>();
    module_registry.add_module::<LegalVoteModule>();
    module_registry.add_module::<LiveKitModule>();
    module_registry.add_module::<MeetingNotesModule>();
    module_registry.add_module::<MeetingReportModule>();
    module_registry.add_module::<ModerationModule>();
    module_registry.add_module::<EchoModule>();
    module_registry.add_module::<PollsModule>();
    module_registry.add_module::<SharedFolderModule>();
    module_registry.add_module::<SubroomAudioModule>();
    module_registry.add_module::<TimerModule>();
    module_registry.add_module::<TrainingParticipationReportModule>();
    module_registry.add_module::<RaiseHandsModule>();
    module_registry.add_module::<WhiteboardModule>();
    module_registry
}

/// Context for the API endpoints
#[derive(Clone)]
pub(crate) struct Context {
    settings: Arc<Settings>,
    /// Global list of room tasks and their handles
    room_tasks: RoomTaskRegistry<WebSocketAdapter>,
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
                room_parameters.into(),
                Arc::clone(&self.module_registry),
                Arc::clone(&self.settings),
                self.app_state.subscribe(),
            )
            .await
            .map_err(|err| {
                tracing::info!("Failed to put room {room_id}: {err}");
                err
            })?;

        if !action.is_created() {
            // Refresh the idle timeout if the room was not created with this request
            task_handle.refresh_idle_timeout().await.map_err(|err| {
                tracing::info!("Failed to refresh idle timeout for room {room_id}: {err}");
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
    ) -> Result<RoomServerAccess, ApiError> {
        let task_handle = self.room_tasks.get_task_handle(&room_id).await;

        match (task_handle, room_parameters) {
            // Room doesn't exist and no parameters were provided
            (None, None) => {
                return Err(ApiError::unprocessable_entity()
                    .with_code("room_parameters_missing")
                    .with_message("Room parameters missing"));
            }

            // Room needs to be created
            (None, Some(parameters)) => {
                self.room_tasks
                    .create_if_not_exists(
                        room_id,
                        parameters.into(),
                        Arc::clone(&self.module_registry),
                        Arc::clone(&self.settings),
                        self.app_state.subscribe(),
                    )
                    .await;
            }

            // Room already exists
            (Some(task_handle), _) => {
                // refresh the idle timeout of the room
                if let Err(e) = task_handle.refresh_idle_timeout().await {
                    tracing::error!("Failed to refresh idle timeout of room {room_id}: {e}");
                    return Err(ApiError::internal().with_message("Failed to refresh idle timeout"));
                }

                // Ensure the user isn't banned
                if let Some(user_id) = client_parameters.kind.user_id() {
                    let is_banned = match task_handle.is_banned(user_id).await {
                        Ok(is_banned) => is_banned,
                        Err(e) => {
                            tracing::error!(
                                "Failed to check ban status of participant {user_id} for room {room_id}: {e}"
                            );
                            return Err(ApiError::internal()
                                .with_message("Failed to check the users ban status"));
                        }
                    };

                    if is_banned {
                        return Err(ApiError::forbidden()
                            .with_code("banned")
                            .with_message("User is banned from this room"));
                    }
                }
            }
        }

        let token = self
            .token_store
            .lock()
            .await
            .create_token(SignalingClientContext::new(room_id, client_parameters));

        let public_url = self.settings.http.public_url.clone();

        Ok(RoomServerAccess { public_url, token })
    }
}

#[cfg(test)]
mod test {
    use std::{borrow::Cow, sync::Arc, time::Duration};

    use axum::http::StatusCode;
    use opentalk_roomserver_types::{
        client_parameters::{ClientKind, Role},
        module_settings::ModuleSettings,
        public_user_profile::PublicUserProfile,
        room_parameters::AssetStorageConfig,
    };
    use opentalk_types_api_v1::error::ErrorBody;
    use opentalk_types_common::{
        roomserver::DeviceSecret,
        tariffs::TariffResource,
        time::TimeZone,
        users::{DisplayName, UserId, UserInfo, UserTitle},
        utils::ExampleData,
    };
    use pretty_assertions::assert_eq;

    use super::*;

    fn test_context() -> Context {
        let settings: Arc<Settings> = Arc::new(Settings::test_settings("secret".into()));
        let (app_state, _) = watch::channel(ApplicationState::Running);

        Context {
            settings: settings.clone(),
            room_tasks: RoomTaskRegistry::new(Duration::from_secs(10)),
            token_store: Arc::new(Mutex::new(TokenStore::new())),
            module_registry: Arc::new(ModuleRegistry::new()),
            app_state,
        }
    }

    fn client_parameters2() -> ClientParameters {
        ClientParameters {
            device_secret: DeviceSecret::example_data(),
            kind: ClientKind::Guest {
                display_name: DisplayName::from_str_lossy("tester"),
            },
            role: Role::User,
        }
    }

    fn client_parameters1() -> ClientParameters {
        ClientParameters {
            device_secret: DeviceSecret::example_data(),
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
                    timezone: TimeZone::example_data(),
                },
            },
            role: Role::Moderator,
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
            module_settings: ModuleSettings::example_data(),
            asset_storage: AssetStorageConfig::example_data(),
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
            .await;

        assert!(matches!(
            token,
            Err(ApiError {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                body: ErrorBody {
                    code: Cow::Borrowed("room_parameters_missing"),
                    ..
                },
                ..
            })
        ));

        let token = ctx
            .request_room_token(RoomId::nil(), client_parameters1(), Some(room_parameters()))
            .await;

        assert!(token.is_ok());

        let token = ctx
            .request_room_token(RoomId::nil(), client_parameters2(), None)
            .await;

        assert!(token.is_ok())
    }
}
