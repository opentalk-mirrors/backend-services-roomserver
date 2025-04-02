// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{future::pending, sync::Arc, time::Duration};

use futures::stream::{FuturesUnordered, StreamExt};
use opentalk_roomserver_signaling::{
    loopback::LoopbackFuture,
    participant_state::{ParticipantKind, ParticipantState, Participants},
    room_info::RoomInfo,
    signaling_module::{FatalError, SignalingModuleInitData},
};
use opentalk_roomserver_types::{
    client_parameters::{ClientKind, ClientParameters},
    connection_id::ConnectionId,
    device_id::DeviceId,
    error::SignalingError,
    room_parameters::RoomParameters,
};
use opentalk_roomserver_web_api::v1::signaling::websocket::SignalingSocket;
use opentalk_types_common::{modules::ModuleId, rooms::RoomId, time::Timestamp};
use opentalk_types_signaling::ParticipantId;
use tokio::sync::{mpsc, watch};
use uuid::Uuid;

use super::{
    message_router::{AlreadyConnectedError, CloseReason},
    signaling::module_initializer::{ModuleRegistry, Modules},
};
use crate::{
    room::{
        message_router::{MessageEnvelope, MessageRouter, SignalingMessage},
        registry::RoomTaskRegistry,
        signaling::{dyn_module_context::DynModuleContext, DynEvent},
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

    /// Loopback futures that were created by signaling modules
    loopback_futures: FuturesUnordered<LoopbackFuture>,

    settings: Arc<Settings>,

    app_state: watch::Receiver<ApplicationState>,

    participants: Participants,
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

            let loopback_futures: FuturesUnordered<LoopbackFuture> = FuturesUnordered::new();
            loopback_futures.push(Box::pin(pending()));

            let room_task = RoomTask {
                info: room_info,
                api_rx: rx,
                idle_timeout: IdleTimeout::start_new(timeout),
                message_router,
                loopback_futures,
                settings,
                app_state,
                participants: Participants::new(),
            };

            log::debug!("Spawn room with modules: {:?}", modules.keys());
            room_task.run(modules).await;
            task_registry.remove_room(room_id).await;
        });

        RoomTaskHandle { sender: tx }
    }

    async fn run(self, modules: Modules) {
        let room_id = self.info.room_id;

        if let Err(e) = self.inner_run(modules).await {
            log::error!("RoomTask exited with error {e}");
        }

        log::debug!("Closing room {room_id}");
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
                    let mut ctx = self.context(msg.participant_id, msg.connection_id);
                    if let Err(err) = module.on_event(&mut ctx, DynEvent::LoopbackEvent(msg.value)).await {
                        handle_fatal_module_error(&mut ctx, &mut modules, msg.namespace, err).await;
                    }
                },
                () = self.idle_timeout.has_timed_out() => {
                    log::debug!("Room task {} reached its idle timeout, exiting", self.info.room_id);
                    return Ok(());
                }
                result = self.app_state.changed() => {
                    if result.is_err() || self.app_state.borrow().is_shutting_down() {
                        log::debug!("Room task {} received shutdown signal, exiting", self.info.room_id);
                        return Ok(())
                    }

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
        self.connect_participant(socket, modules, client_parameters)
            .await;
        self.idle_timeout.stop();
    }

    #[tracing::instrument(level = "info", skip_all, parent = &span, fields(participant_id = %participant_id))]
    async fn handle_message(
        &mut self,
        modules: &mut Modules,
        MessageEnvelope {
            participant_id,
            connection_id,
            message,
            span,
        }: MessageEnvelope<SignalingMessage>,
    ) -> anyhow::Result<()> {
        match message {
            SignalingMessage::Closed(close_reason) => {
                log::trace!("Websocket closed for participant {participant_id}: {close_reason:?}");
                self.disconnect_participant(modules, participant_id, connection_id, close_reason)
                    .await;
            }

            SignalingMessage::Command(signaling_command) => {
                log::trace!("received signaling command: {signaling_command:?}");

                let Some(module) = modules.get_mut(&signaling_command.namespace) else {
                    self.handle_unknown_namespace(
                        participant_id,
                        connection_id,
                        signaling_command.namespace,
                    )
                    .await;
                    return Ok(());
                };

                let mut ctx = self.context(participant_id, connection_id);

                if let Err(err) = module
                    .on_event(
                        &mut ctx,
                        DynEvent::WebsocketMessage {
                            participant_id,
                            connection_id,
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

    async fn connect_participant(
        &mut self,
        socket: Socket,
        modules: &mut Modules,
        client_parameters: ClientParameters,
    ) {
        let device_id = self.derive_device_id(&client_parameters.device_secret);

        let (participant_id, display_name, kind) = match &client_parameters.kind {
            ClientKind::Registered { profile } => (
                ParticipantId::from(Uuid::from(profile.id)),
                profile.user_info.display_name.clone(),
                ParticipantKind::User,
            ),

            ClientKind::Guest { display_name } => {
                let participant_id = ParticipantId::from(Uuid::from(device_id));

                (participant_id, display_name.clone(), ParticipantKind::Guest)
            }
        };

        // If we ever run into the issue of an uuid collision, a guest could hijack a user session and vice versa. We'd
        // rather decline the new connection when the participant id is known, but the participant kinds differ.
        if let Some(existing_participant) = self.participants.all.get(&participant_id) {
            if existing_participant.kind != kind {
                log::error!("ParticipantId collision, dropping new participant ({participant_id})");
                return;
            }
        };

        let connection_id = match self
            .message_router
            .add_connection(participant_id, socket)
            .await
        {
            Ok(conn_id) => conn_id,
            Err(AlreadyConnectedError) => {
                log::debug!("rejecting participant connection: already connected");
                return;
            }
        };

        if let Err(err) = core::participant_joined(
            &mut self.context(participant_id, connection_id),
            participant_id,
            connection_id,
            modules,
            client_parameters,
        )
        .await
        {
            log::error!("failed to add participant to conference {err:#?}");

            self.disconnect_participant(
                modules,
                participant_id,
                connection_id,
                CloseReason::InternalError,
            )
            .await;
        }

        self.participants
            .all
            .entry(participant_id)
            .or_insert_with(|| ParticipantState::new(display_name, kind))
            .connections
            .insert(connection_id, device_id);
    }

    async fn disconnect_participant(
        &mut self,
        modules: &mut Modules,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        reason: CloseReason,
    ) {
        let Some(state) = self.participants.all.get_mut(&participant_id) else {
            log::error!("Attempted to disconnect participant who does not exist");
            return;
        };
        state.connections.remove(&connection_id);

        let ctx = &mut self.context(participant_id, connection_id);

        core::participant_disconnected(ctx, reason.into(), modules).await;

        // start idle timeout when no one is connected
        if !self.participants.all.values().any(|s| s.is_connected()) {
            self.idle_timeout.start(IDLE_TIMEOUT);
        }
    }

    async fn handle_unknown_namespace(
        &mut self,
        origin: ParticipantId,
        connection_id: ConnectionId,
        namespace: ModuleId,
    ) {
        log::debug!(
            "Received signaling message with unknown namespace: {}",
            &namespace
        );

        let signaling_error = SignalingError::UnknownNamespace {
            invalid_namespace: namespace.to_string(),
        };

        self.context(origin, connection_id)
            .send_ws_error(signaling_error)
            .await;
    }

    fn context(
        &mut self,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> DynModuleContext<'_> {
        DynModuleContext::new(
            self.info.room_id,
            participant_id,
            connection_id,
            &mut self.info,
            &mut self.message_router,
            &mut self.participants,
            &self.loopback_futures,
        )
    }

    /// Generate a [`DeviceId`] from a device secret
    ///
    /// This function hashes the device secret and the salt that is configured for the roomserver. The first 128
    /// bit of the output hash are then used as the uuid for the [`DeviceId`]. This is repeatable, the same device
    /// secret will result in the same [`DeviceId`] until the salt changes because of a roomserver restart.
    ///
    /// Reusing the salt is fine in this case since the salt is private and the device secret already has a high entropy.
    /// In contrast to a password salt, our salt needs to stay private.
    fn derive_device_id(&self, device_secret: &str) -> DeviceId {
        let mut hasher = blake3::Hasher::new();
        let salt = self.settings.conference.signaling_salt.as_bytes();
        hasher.update(salt);
        hasher.update(device_secret.as_bytes());

        let mut uuid_bytes = [0; 16];

        hasher.finalize_xof().fill(&mut uuid_bytes);

        DeviceId::from(Uuid::from_bytes(uuid_bytes))
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
