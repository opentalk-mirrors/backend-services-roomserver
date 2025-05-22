// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! The [`RoomTask`] is the central component of a meeting room. It initializes
//! the [`MessageRouter`] and [`SignalingModule`]s. The [`RoomTask`] also
//! * accepts new participants and adds them to the [`MessageRouter`]
//! * forwards messages from the [`MessageRouter`] to the [`SignalingModule`]s
//! * forwards [`LoopbackMessage`]s between the [`SignalingModule`]s
//!
//! ```text
//! ┌──────────┐
//! │ RoomTask │
//! └──┬─┬─────┘
//!    │ └──────────────┐
//!    │                │
//! ┌──▼────────────┐ ┌─▼───────┐
//! │ MessageRouter │ │ Modules │
//! └───────────────┘ └─┬───────┘
//!                     │
//!                   ┌─▼────────────┐
//!                   │   <Trait>    │
//!                   │ ModuleHandle │
//!                   └─▲ ───────────┘
//!                     │
//!                    Implements
//!                     │
//!                   ┌─┴────────────────┐
//!                   │ ModuleDispatcher │
//!                   └─┬────────────────┘
//!                     │
//!                   ┌─▼───────────────┐
//!                   │     <Trait>     │
//!                   │ SignalingModule │
//!                   └─────────────────┘
//! ```
//!
//! [`SignalingModule`]: opentalk_roomserver_signaling::signaling_module::SignalingModule

use std::{future::pending, sync::Arc, time::Duration};

use breakout::state::BreakoutState;
use futures::stream::{FuturesUnordered, StreamExt};
use opentalk_roomserver_common::{application_state::ApplicationState, settings::Settings};
use opentalk_roomserver_signaling::{
    loopback::{LoopbackFuture, LoopbackMessage},
    participant_state::{ParticipantKind, ParticipantState, Participants},
    room_info::RoomInfo,
    signaling_module::SignalingModuleInitData,
};
use opentalk_roomserver_types::{
    breakout_id::BreakoutId,
    client_parameters::{ClientKind, ClientParameters},
    connection_id::ConnectionId,
    device_id::DeviceId,
    error::SignalingError,
    room_parameters::RoomParameters,
};
use opentalk_roomserver_web_api::v1::signaling::websocket::SignalingSocket;
use opentalk_types_common::{rooms::RoomId, time::Timestamp};
use opentalk_types_signaling::ParticipantId;
use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
};
use uuid::Uuid;

use super::{
    message_router::{AlreadyConnectedError, CloseReason},
    signaling::module_initializer::{ModuleRegistry, Modules},
};
use crate::{
    message_router::{MessageEnvelope, MessageRouter, SignalingMessage},
    signaling::{DynEvent, dyn_module_context::DynModuleContext},
    task::{
        handle::{Request, RoomTaskHandle, TaskMessage},
        idle_timeout::IdleTimeout,
    },
};

pub mod breakout;
pub mod core;
pub mod handle;
pub mod idle_timeout;

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
pub struct RoomTask<Socket: SignalingSocket + 'static> {
    info: RoomInfo,

    /// The receiver for web server API request that target this room
    api_rx: mpsc::Receiver<TaskMessage<Socket>>,

    /// The rooms idle timeout, only active when no participants are in the room.
    idle_timeout: IdleTimeout,

    message_router: MessageRouter,

    breakout_config: Option<BreakoutState>,

    /// Loopback futures that were created by signaling modules
    loopback_futures: FuturesUnordered<LoopbackFuture>,

    settings: Arc<Settings>,

    app_state: watch::Receiver<ApplicationState>,

    participants: Participants,

    modules: Modules,
}

impl<Socket: SignalingSocket> RoomTask<Socket> {
    /// Spawns a new [`RoomTask`]
    #[tracing::instrument(level = "debug", skip_all, fields(opentalk.room_id = %room_id))]
    pub fn spawn(
        room_id: RoomId,
        room_parameters: RoomParameters,
        module_registry: Arc<ModuleRegistry>,
        settings: Arc<Settings>,
        app_state: watch::Receiver<ApplicationState>,
    ) -> (RoomTaskHandle<Socket>, JoinHandle<()>) {
        Self::spawn_with_timeout(
            room_id,
            room_parameters,
            app_state,
            module_registry,
            settings,
            IDLE_TIMEOUT,
        )
    }

    /// Spawns a new [`RoomTask`] with a specific timeout
    #[tracing::instrument(level = "info", skip_all, fields(opentalk.room_id = %room_id))]
    pub fn spawn_with_timeout(
        room_id: RoomId,
        mut room_parameters: RoomParameters,
        app_state: watch::Receiver<ApplicationState>,
        module_registry: Arc<ModuleRegistry>,
        settings: Arc<Settings>,
        timeout: Duration,
    ) -> (RoomTaskHandle<Socket>, JoinHandle<()>) {
        let (tx, rx) = mpsc::channel(20);

        let message_router = MessageRouter::new(app_state.clone());

        let join_handle = tokio::task::spawn(async move {
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
                breakout_config: None,
                loopback_futures,
                settings,
                app_state,
                participants: Participants::new(),
                modules,
            };

            room_task.run().await;
        });

        (RoomTaskHandle { sender: tx }, join_handle)
    }

    async fn run(self) {
        log::debug!("Spawn room with modules: {:?}", self.modules.keys());
        let room_id = self.info.room_id;

        if let Err(e) = self.inner_run().await {
            log::error!("RoomTask exited with error {e}");
        }

        log::debug!("Closing room {room_id}");
    }

    async fn inner_run(mut self) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                msg = self.api_rx.recv() => {
                    let Some(msg) = msg else {
                        // TaskHandle dropped, exiting
                        log::warn!("Room tasks {} api channel was dropped, exiting", self.info.room_id);
                        return Ok(());
                    };

                    self.handle_api_request(msg).await?;
                },
                msg = self.message_router.recv() => {
                    log::trace!("received {msg:?}");
                    self.handle_message(msg).await;

                }
                Some(msg) = self.loopback_futures.next() => {
                    self.handle_loopback(msg).await;
                },
                () = self.idle_timeout.has_timed_out() => {
                    log::debug!("Room task {} reached its idle timeout, exiting", self.info.room_id);
                    return Ok(());
                }
                () = Self::check_breakout_timeout(&mut self.breakout_config) => {
                    self.breakout_expired().await;
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
    async fn handle_api_request(&mut self, msg: TaskMessage<Socket>) -> anyhow::Result<()> {
        let api_response = match msg.request {
            Request::RefreshIdleTimeout => {
                self.refresh_idle_timeout();
                Ok(())
            }
            Request::UpdateParameter(room_parameters) => {
                self.update_parameter(*room_parameters);
                Err(RoomTaskApiError::NotImplemented)
            }
            Request::WsJoin {
                socket,
                client_parameters,
            } => {
                self.ws_join(socket, client_parameters).await;
                Ok(())
            }
        };

        let _ = msg.response_channel.send(api_response);

        Ok(())
    }

    #[tracing::instrument(skip_all, fields(opentalk.room_id = %self.info.room_id))]
    async fn handle_loopback(&mut self, msg: Option<LoopbackMessage>) {
        let Some(msg) = msg else {
            log::error!("Signaling module channel was dropped");
            return;
        };
        let Some(module) = self.modules.get_mut(&msg.namespace) else {
            log::error!(
                "Received loopback event for unknown module {}",
                msg.namespace
            );
            return;
        };
        let mut ctx = DynModuleContext::new(
            self.info.room_id,
            msg.breakout_room,
            msg.participant_id,
            msg.connection_id,
            &mut self.info,
            &mut self.message_router,
            &mut self.participants,
            &self.loopback_futures,
        );
        if let Err(err) = module
            .on_event(&mut ctx, DynEvent::LoopbackEvent(msg.value))
            .await
        {
            self.handle_fatal_module_error(msg.namespace, err).await;
        }
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
    async fn ws_join(&mut self, socket: Socket, client_parameters: ClientParameters) {
        self.connect_participant(socket, client_parameters).await;
        self.idle_timeout.stop();
    }

    #[tracing::instrument(level = "info", skip_all, parent = &span, fields(participant_id = %participant_id))]
    async fn handle_message(
        &mut self,
        MessageEnvelope {
            participant_id,
            connection_id,
            message,
            span,
        }: MessageEnvelope<SignalingMessage>,
    ) {
        match message {
            SignalingMessage::Closed(close_reason) => {
                log::trace!("Websocket closed for participant {participant_id}: {close_reason:?}");
                self.disconnect_participant(participant_id, connection_id, close_reason)
                    .await;
            }

            SignalingMessage::Command(signaling_command) => {
                log::trace!("received signaling command: {signaling_command:?}");

                let Some(participant_state) = self.participants.all_unfiltered.get(&participant_id)
                else {
                    log::error!(
                        "failed to get participant state for participant `{participant_id}`"
                    );

                    // This scenario should never occur because we never delete known participants. We still attempt to
                    // send an error to the non-existent connection in a best-effort approach.
                    self.message_router
                        .send_error(connection_id, SignalingError::Internal)
                        .await;

                    return;
                };

                let room_scope = participant_state.breakout_room;

                match &signaling_command.namespace {
                    m if *m == core::NAMESPACE => {
                        log::warn!("🚨🚨🚨 received unsupported core command 🚨🚨🚨");
                        return;
                    }
                    m if *m == breakout::NAMESPACE => {
                        self.handle_breakout_command(
                            participant_id,
                            connection_id,
                            room_scope,
                            signaling_command,
                        )
                        .await;

                        return;
                    }
                    _ => (),
                }

                let Some(module) = self.modules.get_mut(&signaling_command.namespace) else {
                    self.handle_unknown_namespace(
                        connection_id,
                        signaling_command.namespace.to_string(),
                    )
                    .await;

                    return;
                };

                let mut ctx = DynModuleContext::new(
                    self.info.room_id,
                    room_scope,
                    participant_id,
                    connection_id,
                    &mut self.info,
                    &mut self.message_router,
                    &mut self.participants,
                    &self.loopback_futures,
                );

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
                    self.handle_fatal_module_error(signaling_command.namespace, err)
                        .await;
                }
            }
        }
    }

    async fn connect_participant(&mut self, socket: Socket, client_parameters: ClientParameters) {
        let device_id = self.derive_device_id(&client_parameters.device_secret);
        let role = client_parameters.role;

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
        if let Some(existing_participant) = self.participants.all_unfiltered.get(&participant_id) {
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

        self.participants
            .all_unfiltered
            .entry(participant_id)
            .or_insert_with(|| ParticipantState::new(display_name, kind, role))
            .connections
            .insert(connection_id, device_id);

        if let Err(err) = self
            .participant_joined(participant_id, connection_id, client_parameters)
            .await
        {
            log::error!("failed to add participant to conference {err:#?}");

            self.disconnect_participant(participant_id, connection_id, CloseReason::InternalError)
                .await;
        }
    }

    async fn disconnect_participant(
        &mut self,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        reason: CloseReason,
    ) {
        let Some(state) = self.participants.all_unfiltered.get_mut(&participant_id) else {
            log::error!("Attempted to disconnect participant who does not exist");
            return;
        };

        state.connections.remove(&connection_id);

        let breakout_room = state.breakout_room;

        self.participant_disconnected(participant_id, connection_id, breakout_room, reason.into())
            .await;

        // start idle timeout when no one is connected
        if !self
            .participants
            .all_unfiltered
            .values()
            .any(|s| s.is_connected())
        {
            self.idle_timeout.start(IDLE_TIMEOUT);
        }
    }

    async fn handle_unknown_namespace(&mut self, origin: ConnectionId, namespace: String) {
        log::debug!(
            "Received signaling message with unknown namespace: {}",
            &namespace
        );

        let signaling_error = SignalingError::UnknownNamespace {
            invalid_namespace: namespace,
        };

        self.message_router
            .send_error(origin, signaling_error)
            .await;
    }

    fn context(
        &mut self,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        breakout_room: Option<BreakoutId>,
    ) -> DynModuleContext<'_> {
        DynModuleContext::new(
            self.info.room_id,
            breakout_room,
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
