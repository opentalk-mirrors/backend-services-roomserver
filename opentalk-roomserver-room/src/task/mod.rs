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
//! # ConnectionId and ParticipantId
//!
//! Every connection to a Room is identified by the [`ConnectionId`]. The connection ID is generated
//! by [`ScopedRouter::add_connection`](crate::message_router::ScopedRouter::add_connection).
//!
//! For registered users, the [`ParticipantId`] is derived from the [`UserId`] that is part of the
//! [`PublicUserProfile`]. Guests and services don't have a such a profile. These clients provide a
//! `device_secret` that is used to derive a [`DeviceId`] which in turn is used to derive the
//! [`ParticipantId`].
//!
//! [`SignalingModule`]: opentalk_roomserver_signaling::signaling_module::SignalingModule
//! [`UserId`]: opentalk_types_common::users::UserId
//! [`PublicUserProfile`]: opentalk_types_api_v1::users::PublicUserProfile

use std::{
    cell::RefCell,
    collections::{
        HashMap,
        hash_map::Entry::{Occupied, Vacant},
    },
    future::pending,
    mem,
    sync::Arc,
    time::Duration,
};

use anyhow::Context;
use breakout::state::BreakoutState;
use chrono::Utc;
use futures::stream::{FuturesUnordered, StreamExt};
use opentalk_roomserver_common::{application_state::ApplicationState, settings::Settings};
use opentalk_roomserver_signaling::{
    banned_participant::BannedParticipant,
    event_origin::{EventOrigin, ParticipantOrigin},
    instruction::Instruction,
    internal_module_message::InterModuleMessage,
    loopback::{LoopbackFuture, LoopbackMessage},
    module_context::ModuleMessage,
    participant_state::{ParticipantState, Participants},
    room_info::RoomTaskInfo,
    signaling_module::SignalingModuleInitData,
    storage::StorageProvider,
    waiting_participant::WaitingParticipant,
};
use opentalk_roomserver_types::{
    breakout::BREAKOUT_MODULE_ID,
    client_parameters::{ClientKind, ClientParameters, Role},
    connection_id::ConnectionId,
    core::CORE_MODULE_ID,
    device_id::DeviceId,
    error::SignalingError,
    room_kind::RoomKind,
    room_parameters::RoomParameters,
    signaling::SignalingCommand,
};
use opentalk_roomserver_web_api::v1::signaling::websocket::SignalingSocket;
use opentalk_types_common::{
    rooms::RoomId, roomserver::DeviceSecret, tariffs::QuotaType, time::Timestamp,
};
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
    message_router::{MessageEnvelope, MessageRouter, ScopedRouter, SignalingMessage},
    signaling::{DynEvent, dyn_module_context::DynModuleContext},
    task::{
        handle::{Request, RoomTaskHandle, TaskMessage},
        timeout::Timeout,
    },
};

pub mod breakout;
pub mod core;
pub mod fs_storage;
pub mod handle;
pub mod timeout;
pub mod waiting_room;

#[derive(Debug, thiserror::Error)]
pub enum RoomTaskApiError {
    /// Placeholder error for features that are currently missing.
    #[error("This functionality is currently not available")]
    NotImplemented,
}

/// The timeout for an empty room
///
/// Should be higher than the lifetime of the signaling token from the token store to ensure that
/// the room doesn't expire before the signaling token does.
const IDLE_TIMEOUT: Duration = Duration::from_secs(60);

/// The [`RoomTask`] manages the conference state and signaling.
///
/// An idle [`Timeout`] starts when a room has no participants in it. When the idle timeout is
/// reached, the room task exits.
pub struct RoomTask<Socket: SignalingSocket + 'static> {
    info: RoomTaskInfo,

    /// The receiver for web server API request that target this room
    api_rx: mpsc::Receiver<TaskMessage<Socket>>,

    /// The rooms idle timeout, only active when no participants are in the room.
    idle_timeout: Timeout,

    message_router: MessageRouter,

    breakout_config: Option<BreakoutState>,

    /// Loopback futures that were created by signaling modules
    loopback_futures: FuturesUnordered<LoopbackFuture>,

    settings: Arc<Settings>,

    app_state: watch::Receiver<ApplicationState>,

    participants: Participants,

    modules: Modules,

    storage: Arc<dyn StorageProvider>,

    /// Collection of participants in the waiting room.
    waiting_participants: HashMap<ParticipantId, WaitingParticipant>,

    /// Set of participants that are banned from the room
    banned_participants: HashMap<ParticipantId, BannedParticipant>,

    /// Timeout for the room time limit quota
    quota_timeout: Timeout,
}

impl<Socket: SignalingSocket> RoomTask<Socket> {
    /// Spawns a new [`RoomTask`]
    #[tracing::instrument(level = "debug", skip_all, fields(opentalk.room_id = %room_id))]
    pub fn spawn(
        room_id: RoomId,
        room_parameters: Arc<RoomParameters>,
        module_registry: Arc<ModuleRegistry>,
        storage: Arc<dyn StorageProvider>,
        settings: Arc<Settings>,
        app_state: watch::Receiver<ApplicationState>,
    ) -> (RoomTaskHandle<Socket>, JoinHandle<()>) {
        Self::spawn_with_timeout(
            room_id,
            room_parameters,
            app_state,
            module_registry,
            storage,
            settings,
            IDLE_TIMEOUT,
        )
    }

    /// Spawns a new [`RoomTask`] with a specific timeout
    #[tracing::instrument(level = "info", skip_all, fields(opentalk.room_id = %room_id))]
    pub fn spawn_with_timeout(
        room_id: RoomId,
        mut room_parameters: Arc<RoomParameters>,
        app_state: watch::Receiver<ApplicationState>,
        module_registry: Arc<ModuleRegistry>,
        storage: Arc<dyn StorageProvider>,
        settings: Arc<Settings>,
        timeout: Duration,
    ) -> (RoomTaskHandle<Socket>, JoinHandle<()>) {
        let (tx, rx) = mpsc::channel(20);

        let message_router = MessageRouter::new(app_state.clone());

        let join_handle = tokio::task::spawn(async move {
            let (modules, uninitialized) = module_registry
                .initialize_modules(SignalingModuleInitData {
                    settings: Arc::clone(&settings),
                    room_parameters: Arc::clone(&room_parameters),
                })
                .await;

            if !uninitialized.is_empty() {
                // Remove unknown modules from the room parameters
                let mut params = (*room_parameters).clone();
                for module_id in uninitialized {
                    tracing::debug!("Unable to initialize module {module_id} for room {room_id}");
                    params.tariff.modules.remove(&module_id);
                }
                room_parameters = Arc::new(params);
            }

            let room_info = RoomTaskInfo {
                room_id,
                closes_at: room_parameters.calc_time_limit_quota(Timestamp::now()),
                room: (*room_parameters).clone(),
            };

            let loopback_futures: FuturesUnordered<LoopbackFuture> = FuturesUnordered::new();
            loopback_futures.push(Box::pin(pending()));

            let time_limit = room_info
                .room
                .tariff
                .quota(&QuotaType::RoomTimeLimitSecs)
                .unwrap_or(0);

            let room_task = RoomTask {
                info: room_info,
                api_rx: rx,
                idle_timeout: Timeout::start_new(timeout),
                message_router,
                breakout_config: None,
                loopback_futures,
                settings,
                app_state,
                participants: Participants::new(),
                modules,
                storage,
                waiting_participants: HashMap::new(),
                banned_participants: HashMap::new(),
                quota_timeout: Timeout::new(Duration::from_secs(time_limit)),
            };

            room_task.run().await;
        });

        (RoomTaskHandle { sender: tx }, join_handle)
    }

    async fn run(mut self) {
        tracing::debug!("Spawn room with modules: {:?}", self.modules.keys());
        let room_id = self.info.room_id;

        if let Err(e) = self.inner_run().await {
            tracing::error!("RoomTask exited with error {e}");
        }

        tracing::debug!("Shutting down modules");
        for (_, module_handle) in self.modules.drain() {
            module_handle.destroy(room_id);
        }

        tracing::debug!("Closing room {room_id}");
    }

    async fn inner_run(&mut self) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                msg = self.api_rx.recv() => {
                    let Some(msg) = msg else {
                        // TaskHandle dropped, exiting
                        tracing::warn!("Room tasks {} api channel was dropped, exiting", self.info.room_id);
                        return Ok(());
                    };

                    if let Err(e) = self.handle_api_request(msg).await {
                        tracing::error!("Failed to handle room task api request: {e:?}")
                    }
                },
                msg = self.message_router.recv() => {
                    tracing::trace!("received {msg:?}");
                    self.handle_message(msg);

                }
                Some(msg) = self.loopback_futures.next() => {
                    self.handle_loopback(msg);
                },
                () = self.idle_timeout.wait_for_completion() => {
                    tracing::debug!("Room task {} reached its idle timeout, exiting", self.info.room_id);
                    return Ok(());
                }
                () = Self::check_breakout_timeout(&mut self.breakout_config) => {
                    self.breakout_expired();
                }
                result = self.app_state.changed() => {
                    if result.is_err() || self.app_state.borrow().is_shutting_down() {
                        tracing::debug!("Room task {} received shutdown signal, exiting", self.info.room_id);
                        return Ok(())
                    }

                }
                () = self.quota_timeout.wait_for_completion() => {
                    tracing::debug!("Room task {} reached its time limit, exiting", self.info.room_id);
                    self.time_limit_quota_elapsed();
                    return Ok(());
                }
            };
            for (connection_id, participant_id) in self.message_router.disconnected() {
                self.disconnect_participant(
                    EventOrigin::Internal,
                    participant_id,
                    connection_id,
                    CloseReason::ConnectionLost,
                );
            }
        }
    }

    #[tracing::instrument(skip_all, parent = &msg.span, fields(opentalk.room_id = %self.info.room_id))]
    async fn handle_api_request(&mut self, msg: TaskMessage<Socket>) -> anyhow::Result<()> {
        match msg.request {
            Request::RefreshIdleTimeout { response } => {
                self.refresh_idle_timeout();
                response
                    .send(Ok(()))
                    .ok()
                    .context("Failed to respond to RefreshIdleTimeout, response channel dropped")?;
            }
            Request::UpdateParameter {
                response: response_tx,
                parameters,
            } => {
                self.update_parameter(*parameters);
                response_tx
                    .send(Err(RoomTaskApiError::NotImplemented))
                    .ok()
                    .context("Failed to respond to UpdateParameter, response channel dropped")?;
            }
            Request::IsBanned { response, user_id } => {
                let participant_id = ParticipantId::from(Uuid::from(user_id));

                response
                    .send(Ok(self.banned_participants.contains_key(&participant_id)))
                    .ok()
                    .context("Failed to respond to IsBanned, response channel dropped")?;
            }
            Request::WsJoin {
                response,
                socket,
                client_parameters,
            } => {
                self.ws_join(socket, client_parameters);

                response
                    .send(Ok(()))
                    .ok()
                    .context("Failed to respond to WsJoin, response channel dropped")?;
            }
        };

        Ok(())
    }

    fn handle_loopback(&mut self, msg: Option<LoopbackMessage>) {
        let Some(msg) = msg else {
            tracing::error!("Signaling module channel was dropped");
            return;
        };

        let span = msg.span.clone();
        let _enter = span.enter();
        self.handle_loopback_message(msg);
    }

    fn handle_loopback_message(&mut self, msg: LoopbackMessage) {
        let Some(module) = self.modules.get_mut(&msg.namespace) else {
            tracing::error!(
                "Received loopback event for unknown module {}",
                msg.namespace
            );
            return;
        };
        let mut messages = RefCell::new(Vec::new());
        let transaction_id = msg.origin.transaction_id();
        let mut ctx = DynModuleContext::new(
            self.info.room_id,
            msg.room,
            msg.origin,
            &mut self.info,
            &mut self.participants,
            &mut self.waiting_participants,
            &mut self.banned_participants,
            msg.timestamp,
            Arc::clone(&self.storage),
            &mut messages,
            &mut self.loopback_futures,
        );

        let room = ctx.room;
        let origin = ctx.event_origin;
        let timestamp = ctx.timestamp;

        if let Err(err) = module.on_event(&mut ctx, DynEvent::LoopbackEvent(msg.value)) {
            self.handle_fatal_module_error(msg.namespace, transaction_id, err);
        }

        self.handle_module_messages(messages, room, origin, timestamp);
    }

    fn handle_module_messages(
        &mut self,
        messages: RefCell<Vec<ModuleMessage>>,
        room_kind: RoomKind,
        origin: EventOrigin,
        timestamp: Timestamp,
    ) {
        for message in messages.into_inner() {
            match message {
                ModuleMessage::Websocket {
                    connection_id,
                    message,
                } => {
                    self.message_router
                        .conference
                        .send_event([connection_id], message);
                }
                ModuleMessage::WaitingRoomWebsocket {
                    connection_id,
                    message,
                } => self
                    .message_router
                    .waiting_room
                    .send_event([connection_id], message),
                ModuleMessage::InternalCommand(inter_module_message) => {
                    self.handle_internal_command(
                        inter_module_message,
                        room_kind,
                        origin,
                        timestamp,
                    );
                }
                ModuleMessage::Instruction(instruction) => {
                    self.handle_instruction(origin, instruction);
                }
            }
        }
    }

    fn handle_internal_command(
        &mut self,
        command: InterModuleMessage,
        room: RoomKind,
        origin: EventOrigin,
        timestamp: Timestamp,
    ) {
        let Some(module) = self.modules.get_mut(&command.receiver) else {
            tracing::error!(
                "Received internal command for unknown module '{}' from module '{}'",
                command.receiver,
                command.sender,
            );
            return;
        };
        tracing::debug!(
            "Handling internal command from module '{}' to module '{}'",
            command.sender,
            command.receiver
        );

        let mut messages = RefCell::new(Vec::new());
        let mut ctx = DynModuleContext::new(
            self.info.room_id,
            room,
            origin,
            &mut self.info,
            &mut self.participants,
            &mut self.waiting_participants,
            &mut self.banned_participants,
            timestamp,
            Arc::clone(&self.storage),
            &mut messages,
            &mut self.loopback_futures,
        );

        if let Err(err) = module.on_event(
            &mut ctx,
            DynEvent::InternalCommand {
                sender: command.sender,
                command: command.command,
            },
        ) {
            self.handle_fatal_module_error(command.receiver, origin.transaction_id(), err);
        }

        // Disallow modules to trigger internal commands from internal commands
        messages.borrow_mut().retain(|message| match message {
            ModuleMessage::InternalCommand(InterModuleMessage {
                sender, receiver, ..
            }) => {
                tracing::warn!(
                    "Dropping internal command from '{sender}' to '{receiver}' to prevent recursion"
                );
                false
            }
            _ => true,
        });

        self.handle_module_messages(messages, room, origin, timestamp);
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn handle_instruction(&mut self, origin: EventOrigin, instruction: Instruction) {
        match instruction {
            Instruction::Kick { participants } => {
                self.kick_participants(origin, participants);
            }
            Instruction::Ban { participant } => {
                self.ban_participants(origin, participant);
            }
            Instruction::BanWaiting { participant } => {
                self.ban_waiting_participants(participant);
            }
            Instruction::MoveToWaitingRoom { participant } => {
                self.move_to_waiting_room(participant);
            }
        };
    }

    fn kick_participants(&mut self, origin: EventOrigin, participants: Vec<ParticipantId>) {
        for participant_id in participants {
            let Some(state) = self.participants.all_unfiltered.get(&participant_id) else {
                tracing::error!(
                    "Failed to get connections for unknown participant {participant_id}"
                );
                continue;
            };
            let connections: Vec<ConnectionId> = state.connections().collect();
            for connection_id in connections {
                self.disconnect_participant(
                    origin,
                    participant_id,
                    connection_id,
                    CloseReason::Kicked,
                );
            }
        }
    }

    fn ban_participants(&mut self, origin: EventOrigin, participant_id: ParticipantId) {
        let Some(state) = self.participants.all_unfiltered.get_mut(&participant_id) else {
            tracing::error!("Failed to ban participant, {participant_id} does not exist");
            return;
        };

        let connections: Vec<ConnectionId> = state.connections().collect();

        for connection_id in connections {
            self.disconnect_participant(origin, participant_id, connection_id, CloseReason::Banned);
        }
    }

    fn ban_waiting_participants(&mut self, participant_id: ParticipantId) {
        let Some(waiting_participant) = self.waiting_participants.get(&participant_id) else {
            tracing::error!("Failed to ban participant, {participant_id} does not exist");
            return;
        };

        let connections: Vec<ConnectionId> =
            waiting_participant.connections.keys().cloned().collect();

        for connection_id in connections {
            self.disconnect_waiting_participant(participant_id, connection_id);
        }
    }

    #[tracing::instrument(level = "info", skip_all)]
    fn refresh_idle_timeout(&mut self) {
        self.idle_timeout.reset();
    }

    #[tracing::instrument(level = "info", skip(self))]
    fn update_parameter(&mut self, room_parameters: RoomParameters) {
        self.info.room = room_parameters
        // TODO: handle updated values
    }

    #[tracing::instrument(level = "info", skip_all)]
    fn ws_join(&mut self, socket: Socket, client_parameters: ClientParameters) {
        self.connect_participant(socket, client_parameters);
        self.idle_timeout.stop();
    }

    #[tracing::instrument(level = "info", skip_all, parent = &span, fields(participant_id = %participant_id))]
    fn handle_message(
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
                tracing::trace!(
                    "Websocket closed for participant {participant_id}: {close_reason:?}"
                );
                self.handle_disconnect(
                    EventOrigin::Participant(ParticipantOrigin {
                        id: participant_id,
                        connection_id,
                        transaction_id: None,
                    }),
                    participant_id,
                    connection_id,
                    close_reason,
                );
            }

            SignalingMessage::Command(signaling_command) => {
                self.handle_command(signaling_command, participant_id, connection_id);
            }
        }
    }

    fn handle_command(
        &mut self,
        signaling_command: SignalingCommand,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) {
        tracing::trace!("received signaling command: {signaling_command:?}");

        let participant_origin = ParticipantOrigin {
            id: participant_id,
            connection_id,
            transaction_id: signaling_command.transaction_id,
        };

        match &signaling_command.namespace {
            m if *m == CORE_MODULE_ID => {
                self.handle_core_command(participant_origin, signaling_command);
            }
            m if *m == BREAKOUT_MODULE_ID => {
                self.handle_breakout_command(participant_origin, signaling_command);
            }
            _ => {
                self.execute_signaling_module_command(
                    signaling_command,
                    participant_id,
                    connection_id,
                    participant_origin,
                );
            }
        }
    }

    fn execute_signaling_module_command(
        &mut self,
        signaling_command: SignalingCommand,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        participant_origin: ParticipantOrigin,
    ) {
        let room;
        let timestamp = Timestamp::now();
        let mut messages = RefCell::new(Vec::new());
        let origin = participant_origin.into();

        let event = if self.waiting_participants.contains_key(&participant_id) {
            room = RoomKind::Main;
            DynEvent::WaitingRoomWebsocketMessage {
                participant_id,
                connection_id,
                command: signaling_command.payload,
            }
        } else if let Some(participant_state) =
            self.participants.all_unfiltered.get(&participant_id)
        {
            room = participant_state.room;
            DynEvent::WebsocketMessage {
                participant_id,
                connection_id,
                command: signaling_command.payload,
            }
        } else {
            tracing::error!(
                "failed to get participant state for participant `{}`",
                participant_origin.id
            );

            // This scenario should never occur because we never delete known participants. We still
            // attempt to send an error to the non-existent connection in a best-effort
            // approach.
            self.message_router.conference.send_error(
                participant_origin.connection_id,
                signaling_command.transaction_id,
                SignalingError::Internal,
            );

            return;
        };

        let Some(module) = self.modules.get_mut(&signaling_command.namespace) else {
            self.handle_unknown_namespace(
                participant_id,
                connection_id,
                signaling_command.transaction_id,
                signaling_command.namespace.to_string(),
            );

            return;
        };

        let mut ctx = DynModuleContext::new(
            self.info.room_id,
            room,
            origin,
            &mut self.info,
            &mut self.participants,
            &mut self.waiting_participants,
            &mut self.banned_participants,
            timestamp,
            Arc::clone(&self.storage),
            &mut messages,
            &mut self.loopback_futures,
        );
        if let Err(err) = module.on_event(&mut ctx, event) {
            self.handle_fatal_module_error(
                signaling_command.namespace,
                signaling_command.transaction_id,
                err,
            );
        }

        self.handle_module_messages(messages, room, origin, timestamp);
    }

    fn connect_participant(&mut self, socket: Socket, client_parameters: ClientParameters) {
        let device_id = self.derive_device_id(&client_parameters.device_secret);
        let participant_id = build_participant_id(&client_parameters.kind, device_id);
        let role = client_parameters.role;

        // If we ever run into the issue of an uuid collision, a guest could hijack a user session
        // and vice versa. We'd rather decline the new connection when the participant id is
        // known, but the participant kinds differ.
        if let Some(existing_participant) = self.participants.all_unfiltered.get(&participant_id)
            && mem::discriminant(&existing_participant.kind)
                != mem::discriminant(&client_parameters.kind)
        {
            tracing::error!("ParticipantId collision, dropping new participant ({participant_id})");
            return;
        };

        let join_waiting_room = self.info.room.waiting_room
            && !role.is_moderator()
            && self
                .participants
                .all_unfiltered
                .get(&participant_id)
                .map(|participant| participant.in_waiting_room)
                .unwrap_or(true);

        let scoped_router = if join_waiting_room {
            &mut self.message_router.waiting_room
        } else {
            &mut self.message_router.conference
        };
        let connection_id = match scoped_router.add_connection(participant_id, socket) {
            Ok(conn_id) => conn_id,
            Err(AlreadyConnectedError) => {
                tracing::debug!("rejecting participant connection: already connected");
                return;
            }
        };

        if join_waiting_room {
            if let Err(err) =
                self.join_waiting_room(connection_id, participant_id, device_id, client_parameters)
            {
                tracing::error!("failed to add participant to waiting room {err:#?}");

                self.disconnect_waiting_participant(participant_id, connection_id);
            }
        } else {
            self.join_room(
                participant_id,
                connection_id,
                device_id,
                client_parameters.kind,
                client_parameters.role,
            );
        }
    }

    fn join_room(
        &mut self,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        device_id: DeviceId,
        client_kind: ClientKind,
        role: Role,
    ) {
        match self.participants.all_unfiltered.entry(participant_id) {
            Occupied(mut occupied) => {
                let state = occupied.get_mut();
                // Set join/leave timestamps when this is the first device
                if !state.is_connected() {
                    state.joined_at = Utc::now();
                    state.left_at = None;
                }
                state.connections.insert(connection_id, device_id);
            }
            Vacant(vacant) => {
                vacant
                    .insert(ParticipantState::new(
                        client_kind.clone(),
                        role,
                        Utc::now(),
                        false,
                    ))
                    .connections
                    .insert(connection_id, device_id);
            }
        };

        if let Err(err) =
            self.participant_joined(participant_id, connection_id, device_id, client_kind, role)
        {
            tracing::error!("failed to add participant to conference {err:#?}");

            self.disconnect_participant(
                EventOrigin::Internal,
                participant_id,
                connection_id,
                CloseReason::InternalError,
            );
        }
    }

    /// This method either disconnects a waiting room participant or a participant that already
    /// joined the room. For that it either calls [`Self::disconnect_participant`] or
    /// [`Self::disconnect_waiting_participant`].
    fn handle_disconnect(
        &mut self,
        origin: EventOrigin,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        reason: CloseReason,
    ) {
        if self.waiting_participants.contains_key(&participant_id) {
            self.disconnect_waiting_participant(participant_id, connection_id);
        } else {
            self.disconnect_participant(origin, participant_id, connection_id, reason);
        }
    }

    fn disconnect_participant(
        &mut self,
        origin: EventOrigin,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        reason: CloseReason,
    ) {
        let Some(state) = self.participants.all_unfiltered.get_mut(&participant_id) else {
            tracing::error!("Attempted to disconnect participant who does not exist");
            return;
        };

        // When the connection has been removed, the disconnect has already been
        // handled. This is the case when a participant connection has been closed
        // from the server, e.g. when a participant has been kicked. When the
        // connection handle is closed, this function is then called for a second
        // time.
        if state.connections.remove(&connection_id).is_none() {
            return;
        }
        // Set the left_at timestamp if this was the last connection
        if !state.is_connected() {
            state.left_at = Some(Utc::now());
        }

        self.message_router
            .conference
            .remove_connection(connection_id);

        let room = state.room;
        self.participant_disconnected(origin, participant_id, connection_id, room, reason.into());

        // start idle timeout when no one is connected
        if !self
            .participants
            .all_unfiltered
            .values()
            .any(|s| s.is_connected())
        {
            self.idle_timeout.restart();
        }
    }

    fn handle_unknown_namespace(
        &mut self,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        transaction_id: Option<u64>,
        namespace: String,
    ) {
        tracing::debug!(
            "Received signaling message with unknown namespace: {}",
            &namespace
        );

        let signaling_error = SignalingError::UnknownNamespace {
            invalid_namespace: namespace,
        };

        self.message_router_for_participant(participant_id)
            .send_error(connection_id, transaction_id, signaling_error);
    }

    /// Generate a [`DeviceId`] from a device secret
    ///
    /// This function hashes the device secret and the salt that is configured for the roomserver.
    /// The first 128 bit of the output hash are then used as the uuid for the [`DeviceId`].
    /// This is repeatable, the same device secret will result in the same [`DeviceId`] until
    /// the salt changes because of a roomserver restart.
    ///
    /// Reusing the salt is fine in this case since the salt is private and the device secret
    /// already has a high entropy. In contrast to a password salt, our salt needs to stay
    /// private.
    fn derive_device_id(&self, device_secret: &DeviceSecret) -> DeviceId {
        let mut hasher = blake3::Hasher::new();
        let salt = self.settings.conference.signaling_salt.as_bytes();
        hasher.update(salt);
        hasher.update(device_secret.to_string().as_bytes());

        let mut uuid_bytes = [0; 16];

        hasher.finalize_xof().fill(&mut uuid_bytes);

        DeviceId::from(Uuid::from_bytes(uuid_bytes))
    }

    fn message_router_for_participant(
        &mut self,
        participant_id: ParticipantId,
    ) -> &mut ScopedRouter {
        if self.waiting_participants.contains_key(&participant_id) {
            &mut self.message_router.waiting_room
        } else {
            &mut self.message_router.conference
        }
    }
}

fn build_participant_id(kind: &ClientKind, device_id: DeviceId) -> ParticipantId {
    match kind {
        ClientKind::Registered { profile } | ClientKind::RegisteredCallIn { profile } => {
            ParticipantId::from(Uuid::from(profile.id))
        }
        ClientKind::Guest { .. } | ClientKind::Recorder | ClientKind::CallIn { .. } => {
            ParticipantId::from(Uuid::from(device_id))
        }
    }
}
