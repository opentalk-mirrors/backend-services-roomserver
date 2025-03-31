// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    any::Any,
    collections::HashSet,
    future::{pending, Future},
    pin::Pin,
    sync::Arc,
    time::Duration,
};

use futures::stream::{FuturesUnordered, StreamExt};
use opentalk_roomserver_types::{
    client_parameters::ClientParameters, error::SignalingError, room_parameters::RoomParameters,
};
use opentalk_roomserver_web_api::v1::signaling::websocket::SignalingSocket;
use opentalk_types_common::{modules::ModuleId, rooms::RoomId, time::Timestamp};
use opentalk_types_signaling::ParticipantId;
use tokio::sync::{mpsc, watch};

use super::{
    message_router::{AlreadyConnectedError, CloseReason},
    signaling::{
        module_initializer::{ModuleRegistry, Modules},
        signaling_module::{FatalError, SignalingModuleInitData},
    },
};
use crate::{
    room::{
        message_router::{MessageEnvelope, MessageRouter, SignalingMessage},
        registry::RoomTaskRegistry,
        signaling::{module_context::DynModuleContext, DynEvent},
        task::{
            handle::{Request, RoomTaskHandle, TaskMessage},
            idle_timeout::IdleTimeout,
        },
    },
    ApplicationState, Settings,
};

pub(crate) mod core;
pub(crate) mod handle;
mod idle_timeout;

#[derive(Debug, thiserror::Error)]
pub enum RoomTaskApiError {
    /// Placeholder error for features that are currently missing.
    #[error("This functionality is currently not available")]
    NotImplemented,
}

/// The timeout for an empty room
///
/// Should be higher than the lifetime of the signaling token from the token store to ensure that the room doesn't
/// expire before the signaling token does.
const IDLE_TIMEOUT: Duration = Duration::from_secs(60);

pub struct LoopbackMessage {
    pub namespace: ModuleId,
    /// TODO: this might need to be optional at some point
    pub participant_id: ParticipantId,
    pub value: Box<dyn Any + Send + 'static>,
}

/// A set of loopback futures that were created by signaling modules
pub type LoopbackFutures =
    FuturesUnordered<Pin<Box<dyn Future<Output = Option<LoopbackMessage>> + Send + Sync>>>;

/// The [`RoomTask`] manages the conference state and signaling.
///
/// An [`IdleTimeout`] starts when a room has no participants in it. When the idle timeout is reached, the room task
/// exits.
pub(super) struct RoomTask<Socket: SignalingSocket + 'static> {
    info: RoomInfo,

    /// The receiver for web server API request that target this room
    api_rx: mpsc::Receiver<TaskMessage<Socket>>,
    /// The rooms idle timeout, only active when no participants are in the room.
    idle_timeout: IdleTimeout,

    message_router: MessageRouter,

    loopback_futures: LoopbackFutures,

    _settings: Arc<Settings>,

    _app_state: watch::Receiver<ApplicationState>,

    participants: HashSet<ParticipantId>,
}

#[derive(Debug, Clone)]
pub struct RoomInfo {
    /// the identifier of the room
    room_id: RoomId,
    /// The start parameters for the room task
    pub room: RoomParameters,
    /// The time at which the room will close
    pub closes_at: Option<Timestamp>,
}

impl<Socket: SignalingSocket> RoomTask<Socket> {
    /// Spawns a new [`RoomTask`]
    #[tracing::instrument(level = "debug", skip_all, fields(opentalk.room_id = %room_id))]
    pub(super) fn spawn(
        room_id: RoomId,
        room_parameters: RoomParameters,
        task_registry: RoomTaskRegistry<Socket>,
        module_registry: Arc<ModuleRegistry>,
        settings: Arc<Settings>,
        app_state: watch::Receiver<ApplicationState>,
    ) -> RoomTaskHandle<Socket> {
        Self::spawn_with_timeout(
            room_id,
            room_parameters,
            task_registry,
            app_state,
            module_registry,
            settings,
            IDLE_TIMEOUT,
        )
    }

    /// Spawns a new [`RoomTask`] with a specific timeout
    #[tracing::instrument(level = "info", skip_all, fields(opentalk.room_id = %room_id))]
    pub(super) fn spawn_with_timeout(
        room_id: RoomId,
        mut room_parameters: RoomParameters,
        task_registry: RoomTaskRegistry<Socket>,
        app_state: watch::Receiver<ApplicationState>,
        module_registry: Arc<ModuleRegistry>,
        settings: Arc<Settings>,
        timeout: Duration,
    ) -> RoomTaskHandle<Socket> {
        let (tx, rx) = mpsc::channel(20);

        let message_router = MessageRouter::new(app_state.clone());

        tokio::task::spawn(async move {
            let (modules, uninitialized) = module_registry
                .initialize_modules(
                    room_parameters.tariff.modules.keys(),
                    SignalingModuleInitData {
                        settings: Arc::clone(&settings),
                    },
                )
                .await;

            // Remove unknown modules from the room parameters
            for module_id in uninitialized {
                log::debug!("Unable to initialize unknown module {module_id} for room {room_id}");
                room_parameters.tariff.modules.remove(&module_id);
            }

            let room_info = RoomInfo {
                room_id,
                closes_at: room_parameters.calc_time_limit_quota(Timestamp::now()),
                room: room_parameters,
            };

            let loopback_futures = LoopbackFutures::new();
            loopback_futures.push(Box::pin(pending()));

            let room_task = RoomTask {
                info: room_info,
                api_rx: rx,
                idle_timeout: IdleTimeout::start_new(timeout),
                message_router,
                loopback_futures,
                _settings: settings,
                _app_state: app_state,
                participants: HashSet::default(),
            };

            log::debug!("Spawn room with modules: {:?}", modules.keys());
            room_task.run(modules).await;
            task_registry.remove_room(room_id).await;
        });

        RoomTaskHandle { sender: tx }
    }

    async fn run(self, modules: Modules) {
        if let Err(e) = self.inner_run(modules).await {
            log::error!("RoomTask exited with error {e}");
        }
    }

    async fn inner_run(mut self, mut modules: Modules) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                msg = self.api_rx.recv() => {
                    let Some(msg) = msg else {
                        // TaskHandle dropped, exiting
                        log::warn!("Room tasks {} api channel was dropped, exiting", self.info.room_id);
                        return Ok(());
                    };

                    self.handle_api_request(&mut modules, msg).await?;
                },
                msg = self.message_router.recv() => {
                    log::trace!("received {msg:?}");
                    let _ = self.handle_message(&mut modules, msg).await;
                }
                Some(msg) = self.loopback_futures.next() => {
                    let Some(msg) = msg else {
                        log::error!("Signaling module channel was dropped");
                        continue;
                    };
                    let Some(module) = modules.get_mut(&msg.namespace) else {
                        log::error!("Received loopback event for unknown module {}", msg.namespace);
                        continue;
                    };
                    let mut ctx = self.context(msg.participant_id);
                    if let Err(err) = module.on_event(&mut ctx, DynEvent::LoopbackEvent(msg.value)).await {
                        handle_fatal_module_error(&mut ctx, &mut modules, msg.namespace, err).await;
                    }
                },
                () = self.idle_timeout.has_timed_out() => {
                    log::debug!("Room task {} reached its idle timeout, exiting", self.info.room_id);
                    return Ok(());
                }
            };
        }
    }

    #[tracing::instrument(skip_all, parent = &msg.span, fields(opentalk.room_id = %self.info.room_id))]
    async fn handle_api_request(
        &mut self,
        modules: &mut Modules,
        msg: TaskMessage<Socket>,
    ) -> anyhow::Result<()> {
        let api_response = match msg.request {
            Request::RefreshIdleTimeout => {
                self.refresh_idle_timeout();
                Ok(())
            }
            Request::UpdateParameter(room_parameters) => {
                self.update_parameter(room_parameters);
                Err(RoomTaskApiError::NotImplemented)
            }
            Request::WsJoin {
                socket,
                client_parameters,
            } => {
                self.ws_join(modules, socket, client_parameters).await;
                Ok(())
            }
        };

        let _ = msg.response_channel.send(api_response);

        Ok(())
    }

    #[tracing::instrument(level = "info", skip_all)]
    fn refresh_idle_timeout(&mut self) {
        self.idle_timeout.refresh();
    }

    #[tracing::instrument(level = "info", skip(self))]
    fn update_parameter(&mut self, room_parameters: RoomParameters) {
        self.info.room = room_parameters
        // TODO: handle updated values
    }

    #[tracing::instrument(level = "info", skip_all)]
    async fn ws_join(
        &mut self,
        modules: &mut Modules,
        socket: Socket,
        client_parameters: ClientParameters,
    ) {
        self.new_participant(socket, modules, client_parameters)
            .await;
        self.idle_timeout.stop();
    }

    #[tracing::instrument(level = "info", skip_all, parent = &span, fields(participant_id = %participant_id))]
    async fn handle_message(
        &mut self,
        modules: &mut Modules,
        MessageEnvelope {
            participant_id,
            message,
            span,
        }: MessageEnvelope<SignalingMessage>,
    ) -> anyhow::Result<()> {
        match message {
            SignalingMessage::Closed(close_reason) => {
                log::trace!("Websocket closed for participant {participant_id}: {close_reason:?}");
                self.remove_participant(modules, participant_id, close_reason)
                    .await;
            }

            SignalingMessage::Command(signaling_command) => {
                log::trace!("received signaling command: {signaling_command:?}");

                let Some(module) = modules.get_mut(&signaling_command.namespace) else {
                    self.handle_unknown_namespace(participant_id, signaling_command.namespace)
                        .await;
                    return Ok(());
                };

                let mut ctx = self.context(participant_id);

                if let Err(err) = module
                    .on_event(
                        &mut ctx,
                        DynEvent::WebsocketMessage {
                            sender: participant_id,
                            command: signaling_command.content,
                        },
                    )
                    .await
                {
                    handle_fatal_module_error(&mut ctx, modules, signaling_command.namespace, err)
                        .await;
                }
            }
        }

        Ok(())
    }

    async fn new_participant(
        &mut self,
        socket: Socket,
        modules: &mut Modules,
        client_parameters: ClientParameters,
    ) {
        let participant_id = ParticipantId::generate();

        if self
            .message_router
            .register_participant(participant_id, socket)
            .await
            == Err(AlreadyConnectedError)
        {
            log::debug!("rejecting participant connection: already connected");
            return;
        }

        if let Err(err) = core::participant_joined(
            &mut self.context(participant_id),
            participant_id,
            modules,
            client_parameters,
        )
        .await
        {
            log::error!("failed to add participant to conference {err:#?}");

            self.remove_participant(modules, participant_id, CloseReason::InternalError)
                .await;
        }
    }

    async fn remove_participant(
        &mut self,
        modules: &mut Modules,
        participant_id: ParticipantId,
        reason: CloseReason,
    ) {
        self.participants.remove(&participant_id);

        let ctx = &mut self.context(participant_id);

        core::participant_disconnected(ctx, reason.into(), modules).await;

        if self.participants.is_empty() {
            self.idle_timeout.start(IDLE_TIMEOUT);
        }
    }

    async fn handle_unknown_namespace(&mut self, origin: ParticipantId, namespace: ModuleId) {
        log::debug!(
            "Received signaling message with unknown namespace: {}",
            &namespace
        );

        let signaling_error = SignalingError::UnknownNamespace {
            invalid_namespace: namespace.to_string(),
        };

        self.context(origin).send_ws_error(signaling_error).await;
    }

    fn context(&mut self, participant_id: ParticipantId) -> DynModuleContext<'_> {
        DynModuleContext::new(
            self.info.room_id,
            participant_id,
            &mut self.info,
            &mut self.message_router,
            &mut self.participants,
            &self.loopback_futures,
        )
    }
}

/// An unrecoverable module error occurred and the module needs to be removed for the remainder of the conference
///
/// Further requests to the module will result in a [`SignalingError::UnknownNamespace`] error.
pub(crate) async fn handle_fatal_module_error(
    ctx: &mut DynModuleContext<'_>,
    modules: &mut Modules,
    namespace: ModuleId,
    err: FatalError,
) {
    log::error!(
        "The {namespace} module caused a fatal error and will be shut down: {:#?}",
        err.0
    );

    let Some(module) = modules.remove(&namespace) else {
        log::error!("Attempted to remove non-existent module {namespace}");
        return;
    };

    module.destroy().await;

    // Remove the module from the room state
    ctx.room_info.room.tariff.modules.remove(&namespace);

    ctx.broadcast_ws_error(SignalingError::FatalModuleError { namespace })
        .await;
}
