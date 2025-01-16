// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
//! Manages the websocket connection for a single participant
use std::time::Duration;

use futures::{SinkExt as _, StreamExt as _};
use opentalk_roomserver_types::signaling::{SignalingCommand, SignalingError, SignalingEvent};
use opentalk_roomserver_web_api::v1::signaling::websocket::{
    self, CloseFrame, Message, SignalingSocket,
};
use opentalk_types_signaling::ParticipantId;
use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
};

use super::message::{CloseReason, MessageEnvelope, SignalingMessage};
use crate::ApplicationState;

/// The duration the participant has to respond with a close frame after a close
/// frame is sent by the server. If the participant does not send a close frame
/// within this period, the connection will be forcefully terminated.
const CLOSE_TIMEOUT: Duration = Duration::from_secs(5);

const SEND_TIMEOUT: Duration = Duration::from_secs(1);

/// Handle to the task that communicates with the participant ([`ParticipantConnectionTask`]).
///
/// Dropping this handle will close the connection to the participant.
#[derive(Debug)]
pub struct ConnectionHandle {
    /// Event channel to the [`ParticipantConnectionTask`].
    connection_task_event_sender: mpsc::Sender<SignalingEvent>,
}

impl ConnectionHandle {
    /// Instruct the connection task to send an event to the participant.
    ///
    /// If the connection to the participant is already closed or broken, the
    /// event will be dropped and the connection to the participant will be closed.
    ///
    /// ## Timeout
    ///
    /// Sending an event will fail after 1 second. If the participant is
    /// congested, the event is dropped.
    pub async fn send_event(&self, event: SignalingEvent) -> anyhow::Result<()> {
        self.connection_task_event_sender
            .send_timeout(event, SEND_TIMEOUT)
            .await?;

        Ok(())
    }
}

/// The reason for exiting the [`ParticipantConnectionTask`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExitReason {
    /// The room task closed the connection
    ClosedByRoomTask,
    /// The client closed the connection
    ClosedByClient,
    /// The websocket was disconnected for no specific reason
    UnexpectedDisconnection,
    /// The room server is shutting down
    Shutdown,
}

impl ExitReason {
    /// Create a close frame appropriate for the exit reason
    fn close_frame(&self) -> Option<CloseFrame> {
        match self {
            ExitReason::ClosedByClient => None,
            ExitReason::UnexpectedDisconnection => None,

            ExitReason::ClosedByRoomTask => Some(CloseFrame {
                code: 1000,
                reason: "closed by server".into(),
            }),
            ExitReason::Shutdown => Some(CloseFrame {
                code: 1001,
                reason: "server shutdown".into(),
            }),
        }
    }
}

impl From<ExitReason> for CloseReason {
    fn from(value: ExitReason) -> Self {
        match value {
            ExitReason::UnexpectedDisconnection => Self::ConnectionLost,
            ExitReason::Shutdown | ExitReason::ClosedByRoomTask => Self::TaskClosed,
            ExitReason::ClosedByClient => Self::ParticipantClosed,
        }
    }
}

/// Handles the messaging between client and [`RoomTask`](super::super::task::RoomTask)
///
/// Messages that were sent by the client get tagged with the associated participant id and will be forwarded to the
/// room task. Messages from the room task will be forwarded to the client.
pub(super) struct ParticipantConnectionTask<Socket: SignalingSocket> {
    /// The participant id that is used to tag messages that are send to the room task
    participant_id: ParticipantId,
    /// The sender to communicate commands to the room task
    room_task_command_sender: mpsc::Sender<MessageEnvelope<SignalingMessage>>,
    /// The participants websocket connection
    socket: Socket,
    /// The receiver to communicate events from the room task
    room_task_event_receiver: mpsc::Receiver<SignalingEvent>,
    /// The application state watch
    app_state: watch::Receiver<ApplicationState>,
}

impl<Socket: SignalingSocket + 'static> ParticipantConnectionTask<Socket> {
    const EVENT_BUFFER_SIZE: usize = 32;

    /// Create a new [`ParticipantConnectionTask`]
    pub(super) fn create(
        participant_id: ParticipantId,
        socket: Socket,
        room_task_command_sender: mpsc::Sender<MessageEnvelope<SignalingMessage>>,
        app_state: watch::Receiver<ApplicationState>,
    ) -> ConnectionHandle {
        log::debug!("Creating new participant connection task for participant {participant_id}");
        let (event_sender, event_receiver) = mpsc::channel(Self::EVENT_BUFFER_SIZE);

        ParticipantConnectionTask::spawn(
            participant_id,
            room_task_command_sender,
            socket,
            event_receiver,
            app_state,
        );

        ConnectionHandle {
            connection_task_event_sender: event_sender,
        }
    }

    fn spawn(
        participant_id: ParticipantId,
        room_task_command_sender: mpsc::Sender<MessageEnvelope<SignalingMessage>>,
        socket: Socket,
        room_task_event_receiver: mpsc::Receiver<SignalingEvent>,
        app_state: watch::Receiver<ApplicationState>,
    ) -> JoinHandle<()> {
        let this = Self {
            participant_id,
            room_task_command_sender,
            socket,
            room_task_event_receiver,
            app_state,
        };

        tokio::task::spawn(this.run())
    }

    async fn run(mut self) {
        let exit_reason = self.message_loop().await;

        self.perform_close_handshake(exit_reason).await;
    }

    async fn message_loop(&mut self) -> ExitReason {
        loop {
            tokio::select! {
                msg = self.socket.next() => {
                    if let Err(e) = self.handle_websocket_frame(msg).await {
                        return e;
                    }
                },
                event = self.room_task_event_receiver.recv() => {
                    let Some(event) = event else {
                        if self.app_state.borrow().is_shutting_down() {
                            return ExitReason::Shutdown;
                        }

                        return ExitReason::ClosedByRoomTask;
                    };

                    if let Err(e) = self.send_event_to_websocket(&event).await {
                        log::debug!("Failed to send websocket message: {}", e);

                        return ExitReason::UnexpectedDisconnection;
                    }
                }
            }
        }
    }

    async fn handle_websocket_frame(
        &mut self,
        frame: Option<Result<Message, websocket::Error>>,
    ) -> Result<(), ExitReason> {
        match frame {
            Some(Ok(Message::Text(message))) => self.handle_text_frame(message.to_string()).await,
            Some(Ok(Message::Close(close_frame))) => {
                log::debug!(
                    "Connection closed by participant `{:?}`: {close_frame:?}",
                    self.participant_id
                );

                Err(ExitReason::ClosedByClient)
            }
            Some(Ok(_)) => {
                // Ping, Pong and Binary are ignored
                Ok(())
            }
            Some(Err(e)) => {
                log::debug!(
                    "Error while receiving msg from participant `{:?}`: {:?}",
                    self.participant_id,
                    e
                );
                Err(ExitReason::UnexpectedDisconnection)
            }
            None => {
                log::debug!(
                    "Websocket was closed unexpectedly by participant `{:?}`",
                    self.participant_id
                );

                Err(ExitReason::UnexpectedDisconnection)
            }
        }
    }

    async fn handle_text_frame(&mut self, msg: String) -> Result<(), ExitReason> {
        let command = match serde_json::from_str::<SignalingCommand>(&msg) {
            Ok(cmd) => cmd,
            Err(e) => return self.send_error(e).await,
        };
        let wrapped_cmd = SignalingMessage::Command(command).into_envelope(self.participant_id);

        let res = self.room_task_command_sender.send(wrapped_cmd).await;

        match res {
            Ok(_) => Ok(()),
            Err(_) => Err(ExitReason::ClosedByRoomTask),
        }
    }

    async fn send_error(&mut self, error: impl Into<SignalingError>) -> Result<(), ExitReason> {
        let error_message = serde_json::to_string(&error.into())
            .unwrap_or_else(|_| r#"{"error": "server_error"}"#.into())
            .into();

        if let Err(e) = self.socket.send(error_message).await {
            log::debug!(
                "Failed to send error to participant `{}`: {e}",
                self.participant_id
            );
            return Err(ExitReason::UnexpectedDisconnection);
        }

        Ok(())
    }

    /// Parse the [`SignalingEvent`] and send it over the websocket
    async fn send_event_to_websocket(&mut self, event: &SignalingEvent) -> anyhow::Result<()> {
        let message = match serde_json::to_string(event) {
            Ok(message) => message,
            Err(e) => {
                let error_msg =
                    format!("Unable to serialize signaling event to websocket message {e}");

                // This error _should_ never occur. We panic if this is a debug build
                if cfg!(debug_assertions) {
                    panic!("{error_msg}");
                }

                log::error!("{error_msg}");

                return Ok(());
            }
        };

        self.socket.send(message.into()).await?;

        Ok(())
    }

    /// Send a close frame, wait for the close response and drop this task.
    ///
    /// The room task gets notified that the participant has closed the websocket
    async fn perform_close_handshake(self, exit_reason: ExitReason) {
        let Self {
            participant_id,
            room_task_command_sender,
            mut socket,
            ..
        } = self;

        // In case the channel is full, we don't want to wait until we can process this message.
        drop(tokio::spawn(async move {
            // we don't care if the room tasks command receiver was dropped. There is nothing we can do.
            let _ = room_task_command_sender
                .send(
                    SignalingMessage::Closed(CloseReason::from(exit_reason))
                        .into_envelope(participant_id),
                )
                .await;
        }));

        if let Some(close_frame) = exit_reason.close_frame() {
            let _ = socket.send(Message::Close(Some(close_frame))).await;
            // Wait for a close frame for the duration of `CLOSE_TIMEOUT` until we forcefully terminate the connection
            let _ = tokio::time::timeout(CLOSE_TIMEOUT, wait_close(socket)).await;
        }
    }
}

/// Wait for a close frame and discard all other messages
async fn wait_close<Socket: SignalingSocket>(mut socket: Socket) {
    while let Some(msg) = socket.next().await {
        match msg {
            Ok(Message::Close(_)) | Err(_) => return,
            // Discard all messages, but error and close
            _ => {}
        }
    }
}
