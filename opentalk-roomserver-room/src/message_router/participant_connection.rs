// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
//! Manages the websocket connection for a single participant
use std::{pin::Pin, time::Duration};

use futures::{FutureExt, SinkExt as _, StreamExt as _, stream::Peekable};
use opentalk_roomserver_common::application_state::ApplicationState;
use opentalk_roomserver_signaling::signaling_module::SharedRawJson;
use opentalk_roomserver_types::{
    connection_id::ConnectionId, error::SignalingError, signaling::SignalingCommand,
};
use opentalk_roomserver_web_api::v1::signaling::websocket::{
    CloseFrame, Message, SignalingSink, SignalingSocket, SignalingStream,
};
use opentalk_types_signaling::ParticipantId;
use tokio::{
    sync::{
        mpsc::{self, OwnedPermit},
        watch,
    },
    task::JoinHandle,
};

use super::message::{CloseReason, MessageEnvelope, SignalingMessage};

/// The duration the participant has to respond with a close frame after a close
/// frame is sent by the server. If the participant does not send a close frame
/// within this period, the connection will be forcefully terminated.
const CLOSE_TIMEOUT: Duration = Duration::from_secs(5);

/// Timeout for sending messages to the participant.
const SEND_TIMEOUT: Duration = Duration::from_secs(1);

/// The buffer size for events sent to the participant.
const EVENT_BUFFER_SIZE: usize = 32;

/// Handle to the task that communicates with the participant ([`ParticipantConnectionTask`]).
///
/// Dropping this handle will close the connection to the participant.
#[derive(Debug, Clone)]
pub(crate) struct ConnectionHandle {
    /// Event channel to the [`ParticipantConnectionTask`].
    connection_task_event_sender: mpsc::Sender<SharedRawJson>,
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
    pub async fn send_event(&self, event: SharedRawJson) -> anyhow::Result<()> {
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

/// Create a new [`ParticipantConnectionTask`]
pub(super) fn create<Socket: SignalingSocket + 'static>(
    participant_id: ParticipantId,
    connection_id: ConnectionId,
    socket: Socket,
    room_task_command_sender: mpsc::Sender<MessageEnvelope<SignalingMessage>>,
    app_state: watch::Receiver<ApplicationState>,
) -> ConnectionHandle {
    log::debug!("Creating new participant connection task for participant {participant_id}");
    let (event_sender, event_receiver) = mpsc::channel(EVENT_BUFFER_SIZE);

    spawn(
        participant_id,
        connection_id,
        room_task_command_sender,
        socket,
        event_receiver,
        app_state,
    );

    ConnectionHandle {
        connection_task_event_sender: event_sender,
    }
}

fn spawn<Socket: SignalingSocket + 'static>(
    participant_id: ParticipantId,
    connection_id: ConnectionId,
    room_task_command_sender: mpsc::Sender<MessageEnvelope<SignalingMessage>>,
    socket: Socket,
    room_task_event_receiver: mpsc::Receiver<SharedRawJson>,
    app_state: watch::Receiver<ApplicationState>,
) -> JoinHandle<()> {
    let (sink, stream) = socket.split();
    let stream = stream.peekable();

    let this = ParticipantConnectionTask {
        participant_id,
        connection_id,
        room_task_command_sender,
        room_task_event_receiver,
        stream,
        sink,
        app_state,
    };

    tokio::task::spawn(this.run())
}

/// Handles the messaging between client and [`RoomTask`](super::super::task::RoomTask)
///
/// Messages that were sent by the client get tagged with the associated participant id and will be forwarded to the
/// room task. Messages from the room task will be forwarded to the client.
pub(super) struct ParticipantConnectionTask<Stream: SignalingStream, Sink: SignalingSink> {
    /// The participant id that is used to tag messages that are send to the room task
    participant_id: ParticipantId,
    /// The connection id of this specific websocket connection
    connection_id: ConnectionId,
    /// The sender to communicate commands to the room task
    room_task_command_sender: mpsc::Sender<MessageEnvelope<SignalingMessage>>,
    /// The participants websocket connection
    stream: Peekable<Stream>,
    sink: Sink,
    /// The receiver to communicate events from the room task
    room_task_event_receiver: mpsc::Receiver<SharedRawJson>,
    /// The application state watch
    app_state: watch::Receiver<ApplicationState>,
}

impl<Stream: SignalingStream, Sink: SignalingSink> ParticipantConnectionTask<Stream, Sink> {
    async fn run(mut self) {
        let exit_reason = self.message_loop().await;

        self.perform_close_handshake(exit_reason).await;
    }

    /// Loop over a send/receive select until the command channel is closed.
    async fn message_loop(&mut self) -> ExitReason {
        loop {
            let mut stream = Pin::new(&mut self.stream);

            // Build the receive future. We don't bind this to self since this would
            // not allow us to also have a mutable reference in the `self.room_task_event_receiver` (aka send-future).
            let allocated_receive =
                Self::allocated_receive(self.room_task_command_sender.clone(), &mut stream);

            // branches of this select must NOT block
            tokio::select! {
                allocated_msg = allocated_receive => {
                    let (permit, msg) = match allocated_msg {
                        Ok(allocated_msg) => allocated_msg,
                        Err(exit_reason) => return exit_reason,
                    };
                    if let Err(exit_reason) = self.handle_websocket_frame(msg, permit).await{
                        return exit_reason;
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

    /// Receive a message and allocate a slot in the [`RoomTask`]-receive-channel.
    ///
    /// This function ensures that we can actually submit the message to the [`RoomTask`]
    /// and that there is a message to submit.
    ///
    /// ## Cancel Safety
    ///
    /// This function needs to be cancel safe, i.e. when canceled no messages must be dropped.
    /// It is expected that this function will be canceled whenever an event is sent to the participant.
    ///
    /// [`RoomTask`]: crate::room::task::RoomTask
    async fn allocated_receive(
        sender: mpsc::Sender<MessageEnvelope<SignalingMessage>>,
        stream: &mut Pin<&mut Peekable<Stream>>,
    ) -> Result<(OwnedPermit<MessageEnvelope<SignalingMessage>>, Message), ExitReason> {
        let _ = stream.as_mut().peek().await;

        let Ok(permit) = sender.clone().reserve_owned().await else {
            return Err(ExitReason::ClosedByRoomTask);
        };

        // `next`-future must be ready since we already peeked above and received a value
        let Some(frame) = stream.next().now_or_never().flatten() else {
            return Err(ExitReason::UnexpectedDisconnection);
        };
        let Ok(message) = frame else {
            log::debug!("Error while receiving msg from participant: {:?}", frame);
            return Err(ExitReason::UnexpectedDisconnection);
        };

        Ok((permit, message))
    }

    #[tracing::instrument(skip_all, fields(opentalk.participant_id = %self.participant_id))]
    async fn handle_websocket_frame(
        &mut self,
        frame: Message,
        permit: OwnedPermit<MessageEnvelope<SignalingMessage>>,
    ) -> Result<(), ExitReason> {
        match frame {
            Message::Text(message) => self.handle_text_frame(message.to_string(), permit).await,
            Message::Close(close_frame) => {
                log::debug!(
                    "Connection closed by participant `{:?}`: {close_frame:?}",
                    self.participant_id
                );

                Err(ExitReason::ClosedByClient)
            }
            _ => {
                // Ping, Pong and Binary are ignored
                log::debug!("Ignoring ping, pong or binary websocket message");
                Ok(())
            }
        }
    }

    async fn handle_text_frame(
        &mut self,
        msg: String,
        permit: OwnedPermit<MessageEnvelope<SignalingMessage>>,
    ) -> Result<(), ExitReason> {
        let command = match serde_json::from_str::<SignalingCommand>(&msg) {
            Ok(cmd) => cmd,
            Err(e) => {
                log::debug!("Error parsing signaling command: {e}");
                return self.send_error(e).await;
            }
        };
        let wrapped_cmd = SignalingMessage::Command(command)
            .into_envelope(self.connection_id, self.participant_id);

        permit.send(wrapped_cmd);

        Ok(())
    }

    async fn send_error(&mut self, error: impl Into<SignalingError>) -> Result<(), ExitReason> {
        let error_message = serde_json::to_string(&error.into())
            .unwrap_or_else(|_| r#"{"error": "internal"}"#.into())
            .into();

        if let Err(e) = self.sink.send(error_message).await {
            log::debug!(
                "Failed to send error to participant `{}`: {e}",
                self.participant_id
            );
            return Err(ExitReason::UnexpectedDisconnection);
        }

        Ok(())
    }

    /// Parse the [`SignalingEvent`](opentalk_roomserver_signaling::signaling_event::SignalingEvent)
    /// and send it over the websocket
    async fn send_event_to_websocket(&mut self, event: &SharedRawJson) -> anyhow::Result<()> {
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

        self.sink.send(message.into()).await?;

        Ok(())
    }

    /// Send a close frame, wait for the close response and drop this task.
    ///
    /// The room task gets notified that the participant has closed the websocket
    async fn perform_close_handshake(self, exit_reason: ExitReason) {
        let Self {
            participant_id,
            room_task_command_sender,
            mut sink,
            stream,
            ..
        } = self;

        // In case the channel is full, we don't want to wait until we can process this message.
        drop(tokio::spawn(async move {
            // we don't care if the room tasks command receiver was dropped. There is nothing we can do.
            let _ = room_task_command_sender
                .send(
                    SignalingMessage::Closed(CloseReason::from(exit_reason))
                        .into_envelope(self.connection_id, participant_id),
                )
                .await;
        }));

        if let Some(close_frame) = exit_reason.close_frame() {
            let _ = sink.send(Message::Close(Some(close_frame))).await;
            // Wait for a close frame for the duration of `CLOSE_TIMEOUT` until we forcefully terminate the connection
            let _ = tokio::time::timeout(CLOSE_TIMEOUT, wait_close(stream)).await;
        }
    }
}

/// Wait for a close frame and discard all other messages
async fn wait_close<Stream: SignalingStream>(mut stream: Stream) {
    while let Some(msg) = stream.next().await {
        match msg {
            Ok(Message::Close(_)) | Err(_) => return,
            // Discard all messages, but error and close
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::{StreamExt as _, pin_mut};
    use opentalk_roomserver_types::{connection_id::ConnectionId, signaling::SignalingCommand};
    use tokio::sync::mpsc;
    use tracing::Span;

    use crate::{
        message_router::{MessageEnvelope, participant_connection::ParticipantConnectionTask},
        mocking::{mock_socket::MockSocket, participant::create_participant_connection},
    };

    /// Test that the receive future is cancel safe.
    ///
    /// 1. Fill the command channel for the room task
    /// 2. Send a message from the participant though the socket
    /// 3. call `allocated_receive`, which should block at reserving a queue

    #[tokio::test]
    async fn allocated_receive_cancel_safety() {
        const CMD_CHANNEL_SIZE: usize = 10;
        let (command_tx, mut command_rx) = mpsc::channel(CMD_CHANNEL_SIZE);
        let (p1_socket, p1) = create_participant_connection();
        let p1_socket = p1_socket.peekable();
        pin_mut!(p1_socket);

        while command_tx.capacity() > 0 {
            command_tx
                .send(MessageEnvelope {
                    connection_id: ConnectionId::nil(),
                    participant_id: p1.id,
                    message: crate::message_router::SignalingMessage::Command(
                        SignalingCommand::new(
                            "ping".parse().unwrap(),
                            None,
                            serde_json::value::RawValue::NULL.to_owned(),
                        ),
                    ),
                    span: Span::none(),
                })
                .await
                .unwrap();
        }
        let receive_future = ParticipantConnectionTask::<MockSocket, MockSocket>::allocated_receive(
            command_tx.clone(),
            &mut p1_socket,
        );

        // Insert a pending message in the socket, that must not get lost when canceling the receive message
        p1.queue_send_ping();

        let timeout = tokio::time::timeout(Duration::from_millis(100), receive_future).await;
        // must be `Elapsed`, since we can never acquired a Reserve for the command channel
        assert!(
            timeout.is_err(),
            "Task must timeout since there is no space in the channel to reserve"
        );

        // receive all messages from the channel. Make space for the one message we queued earlier.
        let mut buf = Vec::with_capacity(CMD_CHANNEL_SIZE);
        command_rx.recv_many(&mut buf, CMD_CHANNEL_SIZE).await;

        // Receive the queued message, which must not have gotten lost.
        let receive_future = tokio::time::timeout(
            Duration::from_millis(100),
            ParticipantConnectionTask::<MockSocket, MockSocket>::allocated_receive(
                command_tx,
                &mut p1_socket,
            ),
        )
        .await;

        assert!(
            matches!(
                receive_future,
                Ok(Ok((_, axum::extract::ws::Message::Text(_)))),
            ),
            "Receive expired, but a message should be received"
        )
    }
}
