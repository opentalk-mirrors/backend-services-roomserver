// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{collections::HashSet, time::Duration};

use anyhow::Result;
use axum::extract::ws::WebSocket;
use opentalk_roomserver_types::{room_parameters::RoomParameters, signaling::SignalingEvent};
use opentalk_types_common::rooms::RoomId;
use opentalk_types_signaling::ParticipantId;
use tokio::sync::{mpsc, watch};

use super::message_router::AlreadyConnectedError;
use crate::{
    api::ApplicationState,
    room::{
        message_router::{MessageEnvelope, MessageRouter, SignalingMessage},
        registry::RoomTaskRegistry,
        task::{
            handle::{Request, RoomTaskHandle, TaskMessage},
            idle_timeout::IdleTimeout,
        },
    },
};

pub(crate) mod handle;
mod idle_timeout;

#[derive(Debug, thiserror::Error)]
pub enum RoomTaskApiError {
    /// Placeholder error for features that are currently missing.
    #[error("This functionality is currently not available")]
    NotImplemented,
}

const IDLE_TIMEOUT: Duration = Duration::from_secs(30);

/// The [`RoomTask`] manages the conference state and signaling.
///
/// An [`IdleTimeout`] starts when a room has no participants in it. When the idle timeout is reached, the room task
/// exits.
pub(super) struct RoomTask {
    /// the identifier of the room
    room_id: RoomId,
    /// The start parameters for the room task
    parameters: RoomParameters,
    /// The receiver for web server API request that target this room
    api_rx: mpsc::Receiver<TaskMessage>,
    /// The rooms idle timeout, only active when no participants are in the room.
    idle_timeout: IdleTimeout,

    message_router: MessageRouter,

    _app_state: watch::Receiver<ApplicationState>,

    participants: HashSet<ParticipantId>,
}

impl RoomTask {
    /// Spawns a new [`RoomTask`]
    pub(super) fn spawn(
        room_id: RoomId,
        room_parameters: RoomParameters,
        task_registry: RoomTaskRegistry,
        app_state: watch::Receiver<ApplicationState>,
    ) -> RoomTaskHandle {
        let (tx, rx) = mpsc::channel(20);

        let message_router = MessageRouter::new(app_state.clone());

        let room_task = RoomTask {
            room_id,
            parameters: room_parameters,
            api_rx: rx,
            idle_timeout: IdleTimeout::start_new(IDLE_TIMEOUT),
            message_router,
            _app_state: app_state,
            participants: HashSet::default(),
        };

        tokio::task::spawn(async move {
            room_task.run().await;
            task_registry.remove_room(room_id).await;
        });

        RoomTaskHandle { sender: tx }
    }

    async fn run(self) {
        if let Err(e) = self.inner_run().await {
            log::error!("RoomTask exited with error {}", e)
        }
    }

    async fn inner_run(mut self) -> Result<()> {
        // TODO: initialize modules

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
                    let _ = self.handle_message(msg).await;
                }
                _ = self.idle_timeout.has_timed_out() => {
                    log::debug!("Room task {} reached its idle timeout, exiting", self.room_id);
                    return Ok(());
                }
            };
        }
    }

    async fn handle_api_request(&mut self, msg: TaskMessage) -> Result<()> {
        let api_response = match msg.request {
            Request::RefreshIdleTimeout => {
                self.idle_timeout.refresh(IDLE_TIMEOUT);
                Ok(())
            }
            Request::UpdateParameter(room_parameters) => {
                self.parameters = room_parameters;
                // TODO: handle updated values
                Err(RoomTaskApiError::NotImplemented)
            }
            Request::WsJoin { socket } => {
                self.new_participant(socket).await;

                self.idle_timeout.stop();
                Ok(())
            }
        };

        let _ = msg.response_channel.send(api_response);

        Ok(())
    }

    async fn handle_message(
        &mut self,
        MessageEnvelope {
            participant_id,
            message,
        }: MessageEnvelope<SignalingMessage>,
    ) -> anyhow::Result<()> {
        match message {
            SignalingMessage::Closed(close_reason) => {
                log::info!("Websocket closed for participant {participant_id}: {close_reason:?}");

                self.participants.remove(&participant_id);

                if self.participants.is_empty() {
                    self.idle_timeout.start(IDLE_TIMEOUT);
                }
            }
            SignalingMessage::Command(signaling_command) => log::info!(
                "Received command from participant {participant_id}:\n{}\n",
                serde_json::to_string_pretty(&signaling_command).unwrap()
            ),
        }

        self.message_router
            .send_event(
                participant_id,
                SignalingEvent {
                    namespace: "ping".to_string(),
                    content: Default::default(),
                },
            )
            .await;
        Ok(())
    }

    async fn new_participant(&mut self, socket: WebSocket) {
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
}
