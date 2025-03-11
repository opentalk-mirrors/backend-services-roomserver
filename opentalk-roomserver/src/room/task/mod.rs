// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{collections::HashSet, sync::Arc, time::Duration};

use opentalk_roomserver_types::room_parameters::RoomParameters;
use opentalk_roomserver_web_api::v1::signaling::websocket::SignalingSocket;
use opentalk_types_common::rooms::RoomId;
use opentalk_types_signaling::ParticipantId;
use tokio::sync::{mpsc, watch};

use super::{
    message_router::AlreadyConnectedError,
    signaling::{
        module_initializer::{ModuleRegistry, Modules},
        signaling_module::SignalingModuleInitData,
    },
};
use crate::{
    room::{
        message_router::{MessageEnvelope, MessageRouter, SignalingMessage},
        registry::RoomTaskRegistry,
        signaling::module_context::DynModuleContext,
        task::{
            handle::{Request, RoomTaskHandle, TaskMessage},
            idle_timeout::IdleTimeout,
        },
    },
    ApplicationState, Settings,
};

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
    /// the identifier of the room
    room_id: RoomId,
    /// The start parameters for the room task
    parameters: RoomParameters,
    /// The receiver for web server API request that target this room
    api_rx: mpsc::Receiver<TaskMessage<Socket>>,
    /// The rooms idle timeout, only active when no participants are in the room.
    idle_timeout: IdleTimeout,

    message_router: MessageRouter,

    _settings: Arc<Settings>,

    _app_state: watch::Receiver<ApplicationState>,

    participants: HashSet<ParticipantId>,
}

impl<Socket: SignalingSocket> RoomTask<Socket> {
    /// Spawns a new [`RoomTask`]
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

            let room_task = RoomTask {
                room_id,
                parameters: room_parameters,
                api_rx: rx,
                idle_timeout: IdleTimeout::start_new(timeout),
                message_router,
                _settings: settings,
                _app_state: app_state,
                participants: HashSet::default(),
            };

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
                        log::warn!("Room tasks {} api channel was dropped, exiting", self.room_id);
                        return Ok(());
                    };

                    self.handle_api_request(msg).await?;
                },
                msg = self.message_router.recv() => {
                    log::trace!("received {msg:?}");
                    let _ = self.handle_message(&mut modules, msg).await;
                }
                () = self.idle_timeout.has_timed_out() => {
                    log::debug!("Room task {} reached its idle timeout, exiting", self.room_id);
                    return Ok(());
                }
            };
        }
    }

    #[tracing::instrument(skip_all, fields(%self.room_id))]
    async fn handle_api_request(&mut self, msg: TaskMessage<Socket>) -> anyhow::Result<()> {
        let api_response = match msg.request {
            Request::RefreshIdleTimeout => {
                self.refresh_idle_timeout();
                Ok(())
            }
            Request::UpdateParameter(room_parameters) => {
                self.update_parameter(room_parameters);
                Err(RoomTaskApiError::NotImplemented)
            }
            Request::WsJoin { socket } => {
                self.ws_join(socket).await;
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
        self.parameters = room_parameters
        // TODO: handle updated values
    }

    #[tracing::instrument(level = "info", skip_all)]
    async fn ws_join(&mut self, socket: Socket) {
        self.new_participant(socket).await;
        self.idle_timeout.stop();
    }

    async fn handle_message(
        &mut self,
        modules: &mut Modules,
        MessageEnvelope {
            participant_id,
            message,
        }: MessageEnvelope<SignalingMessage>,
    ) -> anyhow::Result<()> {
        match message {
            SignalingMessage::Closed(close_reason) => {
                log::trace!("Websocket closed for participant {participant_id}: {close_reason:?}");

                self.participants.remove(&participant_id);

                if self.participants.is_empty() {
                    self.idle_timeout.start(IDLE_TIMEOUT);
                }
            }
            SignalingMessage::Command(signaling_command) => {
                log::trace!("received signaling command: {signaling_command:?}");

                let Some(module) = modules.get_mut(&signaling_command.namespace) else {
                    log::debug!("Unknown namespace: {}", &signaling_command.namespace);
                    return Ok(());
                };

                module
                    .on_event(
                        &mut self.context(),
                        participant_id,
                        signaling_command.content,
                    )
                    .await?;
            }
        }

        Ok(())
    }

    async fn new_participant(&mut self, socket: Socket) {
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

        self.participants.insert(participant_id);
    }

    fn context(&mut self) -> DynModuleContext<'_> {
        DynModuleContext::new(
            self.room_id,
            &self.parameters,
            &mut self.message_router,
            &self.participants,
        )
    }
}
