// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! Manage participant connection tasks

use std::collections::{hash_map::Entry, HashMap};

use axum::extract::ws::{close_code, CloseFrame};
use futures::SinkExt;
pub use message::{MessageEnvelope, SignalingMessage};
use opentalk_roomserver_types::signaling::SignalingEvent;
use opentalk_roomserver_web_api::v1::signaling::websocket::SignalingSocket;
use opentalk_types_signaling::ParticipantId;
use tokio::sync::{mpsc, watch};

use crate::{room::message_router::participant_connection::ConnectionHandle, ApplicationState};

mod message;
mod participant_connection;

/// Error that is returned when a new participant is registered with the [`MessageRouter`], but the
/// participant ID already has a connection.
#[derive(Debug, Clone, Copy, thiserror::Error, PartialEq, Eq)]
#[error("The participant already has an active connection")]
pub struct AlreadyConnectedError;

/// The message router for managing signaling connections
///
/// Provides the interface for communication between client and [`RoomTask`](super::task::RoomTask)
pub struct MessageRouter {
    /// An internal sender that is given to each [`ParticipantConnectionTask`] to communicate with the [`RoomTask`](super::task::RoomTask)
    ///
    /// [`ParticipantConnectionTask`]: participant_connection::ParticipantConnectionTask
    room_task_command_sender: mpsc::Sender<MessageEnvelope<SignalingMessage>>,

    /// A map of participants and their associated websocket connection
    connections: HashMap<ParticipantId, ConnectionHandle>,

    /// The internal receiver for [`room_task_command_sender`](MessageRouter::room_task_command_sender) that contains
    /// messages for the room task. Can be read through the [`recv`](MessageRouter::recv) method.
    room_task_command_receiver: mpsc::Receiver<MessageEnvelope<SignalingMessage>>,

    /// The global application state
    app_state: watch::Receiver<ApplicationState>,
}

impl MessageRouter {
    const COMMAND_CHANNEL_BUFFER_SIZE: usize = 128;

    /// Create a new [`MessageRouter`]
    pub fn new(app_state: watch::Receiver<ApplicationState>) -> Self {
        let (command_channel, command_egress) = mpsc::channel(Self::COMMAND_CHANNEL_BUFFER_SIZE);

        Self {
            room_task_command_sender: command_channel,
            connections: Default::default(),
            room_task_command_receiver: command_egress,
            app_state,
        }
    }

    /// Send a message to a participant
    pub async fn send_event(&mut self, participant_id: ParticipantId, event: SignalingEvent) {
        let Some(connection_handle) = self.connections.get(&participant_id) else {
            return;
        };

        if connection_handle.send_event(event).await.is_err() {
            log::info!("Attempted to message participant who has already left");
            self.connections.remove(&participant_id);
        }
    }

    /// Receive the next message from any connected participant
    pub async fn recv(&mut self) -> MessageEnvelope<SignalingMessage> {
        // This should never return `None`, the message router holds the sender for this receiver
        let msg = self
            .room_task_command_receiver
            .recv()
            .await
            .expect("internal room_task_channel was closed");

        if matches!(msg.message, SignalingMessage::Closed(_)) {
            self.connections.remove(&msg.participant_id);
        }

        msg
    }

    /// Register a new participant connection
    ///
    /// Spawns a new [`ParticipantConnectionTask`] that manages the websocket connection
    ///
    /// [`ParticipantConnectionTask`]: participant_connection::ParticipantConnectionTask
    pub(crate) async fn register_participant<Socket: SignalingSocket + 'static>(
        &mut self,
        participant_id: ParticipantId,
        mut websocket: Socket,
    ) -> Result<(), AlreadyConnectedError> {
        let entry = self.connections.entry(participant_id);
        let Entry::Vacant(vacant) = entry else {
            let _ = websocket
                .send(axum::extract::ws::Message::Close(Some(CloseFrame {
                    code: close_code::POLICY,
                    reason: "user already connected".into(),
                })))
                .await;
            return Err(AlreadyConnectedError);
        };

        let task_handle = participant_connection::create(
            participant_id,
            websocket,
            self.room_task_command_sender.clone(),
            self.app_state.clone(),
        );

        vacant.insert(task_handle);

        Ok(())
    }

    /// Disconnect the participants websocket
    ///
    /// Returns `true` if the participant existed
    #[allow(dead_code)]
    pub(crate) fn disconnect_participant(&mut self, participant_id: ParticipantId) -> bool {
        // Dropping the participants websocket task handle will signal the websocket task to disconnect
        self.connections.remove(&participant_id).is_some()
    }
}

#[cfg(test)]
mod tests {
    use axum::extract::ws::CloseFrame;
    use futures::SinkExt;
    use opentalk_roomserver_types::signaling::SignalingEvent;
    use opentalk_roomserver_web_api::v1::signaling::websocket::Message;
    use serde_json::json;
    use tokio::sync::watch;

    use crate::{
        mocking::participant::create_participant_connection,
        room::message_router::{self, MessageEnvelope, MessageRouter, SignalingMessage},
        ApplicationState,
    };

    #[tokio::test]
    async fn participant_lifecycle() {
        let (_app_state_send, app_state_recv) = watch::channel(ApplicationState::Running);
        let mut router = MessageRouter::new(app_state_recv);
        let (p1_socket, mut p1) = create_participant_connection();

        assert_eq!(Ok(()), router.register_participant(p1.id, p1_socket).await);

        assert_eq!(
            Ok(()),
            p1.sender
                .send(Ok(Message::Close(Some(CloseFrame {
                    code: 1006,
                    reason: "this is a test".to_string().into(),
                }))))
                .await
        );

        let received = router.recv().await;
        assert_eq!(
            received,
            MessageEnvelope {
                participant_id: p1.id,
                message: SignalingMessage::Closed(
                    message_router::message::CloseReason::ParticipantClosed
                )
            }
        );
        router
            .send_event(
                p1.id,
                SignalingEvent {
                    namespace: "ping".to_string(),
                    content: json!({
                        "cool": 12,
                        "thing": true,
                    }),
                },
            )
            .await;
    }
}
