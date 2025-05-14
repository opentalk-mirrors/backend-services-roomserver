// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! Manage participant connection tasks

use std::collections::{HashMap, hash_map::Entry};

use axum::extract::ws::{CloseFrame, close_code};
use futures::SinkExt;
pub use message::{CloseReason, MessageEnvelope, SignalingMessage};
use opentalk_roomserver_common::application_state::ApplicationState;
use opentalk_roomserver_signaling::signaling_module::SharedRawJson;
use opentalk_roomserver_types::connection_id::ConnectionId;
use opentalk_roomserver_web_api::v1::signaling::websocket::SignalingSocket;
use opentalk_types_signaling::ParticipantId;
use tokio::sync::{Mutex, mpsc, watch};

use crate::message_router::participant_connection::ConnectionHandle;

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

    /// A collection of active websocket connections
    connections: Mutex<HashMap<ConnectionId, ConnectionHandle>>,

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
            connections: Mutex::new(HashMap::new()),
            room_task_command_receiver: command_egress,
            app_state,
        }
    }

    pub async fn add_connection<S: SignalingSocket + 'static>(
        &mut self,
        participant_id: ParticipantId,
        mut websocket: S,
    ) -> Result<ConnectionId, AlreadyConnectedError> {
        let connection_id = ConnectionId::generate();

        let entry = self.connections.get_mut().entry(connection_id);
        let Entry::Vacant(vacant) = entry else {
            tokio::task::spawn(async move {
                websocket
                    .send(axum::extract::ws::Message::Close(Some(CloseFrame {
                        code: close_code::ABNORMAL,
                        reason: "UUID collision, please retry".into(),
                    })))
                    .await
            });

            return Err(AlreadyConnectedError);
        };

        let task_handle = participant_connection::create(
            participant_id,
            connection_id,
            websocket,
            self.room_task_command_sender.clone(),
            self.app_state.clone(),
        );

        vacant.insert(task_handle);

        Ok(connection_id)
    }

    /// Send a [`SignalingEvent`](opentalk_roomserver_signaling::signaling_event::SignalingEvent)
    /// to a participant
    pub async fn send_event(
        &self,
        participant_connections: impl IntoIterator<Item = ConnectionId>,
        event: SharedRawJson,
    ) {
        let mut connections = self.connections.lock().await;

        for id in participant_connections {
            let Some(handle) = connections.get(&id) else {
                log::debug!("failed to get connection handle, connection does not exist");
                continue;
            };

            if handle.send_event(event.clone()).await.is_err() {
                log::debug!("failed to message participant, connection is closed");
                connections.remove(&id);
            }
        }
    }

    /// Send a [`SignalingEvent`](opentalk_roomserver_signaling::signaling_event::SignalingEvent)
    /// to **all** participants
    pub async fn broadcast_event(&self, event: SharedRawJson) {
        let mut connections = self.connections.lock().await;

        let mut send_futures = Vec::new();

        for (connection_id, connection_handle) in &mut *connections {
            let cloned_event = event.clone();

            send_futures.push(async move {
                if connection_handle.send_event(cloned_event).await.is_err() {
                    log::debug!("Attempted to message participant who has already left");
                    return Some(*connection_id);
                }
                None
            });
        }

        // send events to all participants and collect stale connections
        let stale_connections = futures::future::join_all(send_futures).await;

        // remove all stale connections
        for participant_id in stale_connections.iter().flatten() {
            connections.remove(participant_id);
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
            self.connections.get_mut().remove(&msg.connection_id);
        }

        msg
    }
}

#[cfg(test)]
mod tests {
    use axum::extract::ws::CloseFrame;
    use opentalk_roomserver_common::application_state::ApplicationState;
    use opentalk_roomserver_signaling::signaling_event::SignalingEvent;
    use opentalk_roomserver_web_api::v1::signaling::websocket::Message;
    use opentalk_types_common::modules::module_id;
    use opentalk_types_signaling::ParticipantId;
    use serde_json::{json, value::to_raw_value};
    use tokio::sync::watch;

    use crate::{
        message_router::{self, MessageEnvelope, MessageRouter, SignalingMessage},
        mocking::participant::create_participant_connection,
    };

    #[tokio::test]
    async fn participant_lifecycle() {
        let (_app_state_send, app_state_recv) = watch::channel(ApplicationState::Running);
        let mut router = MessageRouter::new(app_state_recv);
        let (p1_socket, p1) = create_participant_connection();
        let p1_id = ParticipantId::from_u128(1);

        let connection = router.add_connection(p1_id, p1_socket).await.unwrap();

        p1.sender
            .send(Ok(Message::Close(Some(CloseFrame {
                code: 1006,
                reason: "this is a test".to_string().into(),
            }))))
            .await
            .unwrap();

        let received = router.recv().await;
        assert!(matches!(
            received,
            MessageEnvelope {
                participant_id,
                connection_id,
                message: SignalingMessage::Closed(
                    message_router::message::CloseReason::ParticipantClosed
                ),
                ..
            } if participant_id == p1_id && connection_id == connection
        ));

        let event = SignalingEvent {
            namespace: module_id!("ping"),
            content: to_raw_value(&json!({
                "cool": 12,
                "thing": true,
            }))
            .unwrap(),
        };
        let shared_json = serde_json::value::to_raw_value(&event).unwrap().into();

        router.send_event([connection], shared_json).await;
    }
}
