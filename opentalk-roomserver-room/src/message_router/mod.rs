// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! Manage participant connection tasks

use std::collections::{HashMap, hash_map::Entry};

use anyhow::Context;
use futures::SinkExt;
pub use message::{CloseReason, MessageEnvelope, SignalingMessage};
use opentalk_roomserver_common::application_state::ApplicationState;
use opentalk_roomserver_signaling::signaling_event::SignalingEvent;
use opentalk_roomserver_types::{
    connection_id::ConnectionId,
    error::{self, SignalingError},
    shared_raw_json::SharedRawJson,
    signaling::module_error::FatalError,
};
use opentalk_roomserver_web_api::v1::signaling::websocket::{
    CloseFrame, SignalingSocket, SignalingSocketMessage,
};
use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::ParticipantId;
use serde::Serialize;
use serde_json::value::RawValue;
use tokio::sync::{Mutex, mpsc, watch};

use crate::message_router::participant_connection::ConnectionHandle;

mod message;
mod participant_connection;

const WS_CLOSE_ABNORMAL: u16 = 1006;

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
                    .send(SignalingSocketMessage::Close(Some(CloseFrame {
                        code: WS_CLOSE_ABNORMAL,
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

    pub fn remove_connection(&mut self, connection_id: ConnectionId) {
        self.connections.get_mut().remove(&connection_id);
    }

    /// Send a [`SignalingEvent`] to a participant
    pub async fn send_event(
        &self,
        participant_connections: impl IntoIterator<Item = ConnectionId>,
        event: SharedRawJson,
    ) {
        let mut connections = self.connections.lock().await;

        for id in participant_connections {
            let Some(handle) = connections.get(&id) else {
                tracing::debug!("failed to get connection handle, connection does not exist");
                continue;
            };

            if handle.send_event(event.clone()).await.is_err() {
                tracing::debug!("failed to message participant, connection is closed");
                connections.remove(&id);
            }
        }
    }

    /// Send a [`SignalingEvent`] to **all** participants
    pub async fn broadcast_event(
        &self,
        event: SharedRawJson,
        excluded_connections: &[ConnectionId],
    ) {
        let mut connections = self.connections.lock().await;

        let mut send_futures = Vec::new();

        for (connection_id, connection_handle) in &mut *connections {
            if excluded_connections.contains(connection_id) {
                continue;
            }

            let cloned_event = event.clone();

            send_futures.push(async move {
                if connection_handle.send_event(cloned_event).await.is_err() {
                    tracing::debug!("Attempted to message participant who has already left");
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

    /// Send a websocket message to the given list of connections
    ///
    /// # Errors
    ///
    /// Returns a [`FatalError`] when the content fails to serialize
    pub(crate) async fn serialize_and_send(
        &self,
        connections: impl IntoIterator<Item = ConnectionId>,
        namespace: ModuleId,
        transaction_id: Option<u64>,
        content: impl Serialize,
    ) -> Result<(), FatalError> {
        let shared_json = Self::serialize_event(namespace, transaction_id, content)?;
        self.send_event(connections, shared_json).await;

        Ok(())
    }

    /// Broadcast a websocket message to all participants
    ///
    /// Returns a [`FatalError`] when the content fails to serialize.
    pub(crate) async fn serialize_and_broadcast(
        &self,
        namespace: ModuleId,
        transaction_id: Option<u64>,
        content: impl Serialize,
    ) -> Result<(), FatalError> {
        let shared_json = Self::serialize_event(namespace, transaction_id, content)?;
        self.broadcast_event(shared_json, &[]).await;

        Ok(())
    }

    /// Broadcast a websocket message to all participants
    ///
    /// Returns a [`FatalError`] when the content fails to serialize.
    pub(crate) async fn serialize_and_broadcast_exclude_connections(
        &self,
        namespace: ModuleId,
        transaction_id: Option<u64>,
        content: impl Serialize,
        excluded_connections: &[ConnectionId],
    ) -> Result<(), FatalError> {
        let shared_json = Self::serialize_event(namespace, transaction_id, content)?;
        self.broadcast_event(shared_json, excluded_connections)
            .await;

        Ok(())
    }

    fn serialize_event(
        namespace: ModuleId,
        transaction_id: Option<u64>,
        content: impl Serialize,
    ) -> Result<SharedRawJson, FatalError> {
        let event = SignalingEvent {
            namespace,
            transaction_id,
            content,
        };
        let shared_json = serde_json::value::to_raw_value(&event)
            .with_context(|| {
                format!(
                    "Failed to serialize message for namespace '{}'",
                    event.namespace
                )
            })
            .map_err(FatalError)?
            .into();

        Ok(shared_json)
    }

    /// Send a websocket error message of type [`SignalingError`] to the associated connection
    ///
    /// The message is always scoped to the [`error::NAMESPACE`]
    pub(crate) async fn send_error(
        &self,
        connection_id: ConnectionId,
        transaction_id: Option<u64>,
        error: SignalingError,
    ) {
        let event = SignalingEvent {
            namespace: error::NAMESPACE,
            transaction_id,
            content: error,
        };
        let shared_json = match serde_json::value::to_raw_value(&event) {
            Ok(value) => value.into(),
            Err(err) => {
                tracing::error!("Failed to serialize SignalingError type: {err}");
                RawValue::from_string(r#"{"error": "internal"}"#.into())
                    .unwrap()
                    .into()
            }
        };

        self.send_event([connection_id], shared_json).await;
    }

    /// Send a websocket error message of type [`SignalingError`] to all participants
    ///
    /// The message is always scoped to the [`error::NAMESPACE`]
    pub(crate) async fn broadcast_error(&self, transaction_id: Option<u64>, error: SignalingError) {
        let event = SignalingEvent {
            namespace: error::NAMESPACE,
            transaction_id,
            content: error,
        };
        let shared_json = match serde_json::value::to_raw_value(&event) {
            Ok(value) => value.into(),
            Err(err) => {
                tracing::error!("Failed to serialize SignalingError type: {err}");
                RawValue::from_string(r#"{"error": "internal"}"#.into())
                    .unwrap()
                    .into()
            }
        };

        self.broadcast_event(shared_json, &[]).await;
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
    use opentalk_roomserver_common::application_state::ApplicationState;
    use opentalk_roomserver_signaling::signaling_event::SignalingEvent;
    use opentalk_roomserver_web_api::v1::signaling::websocket::{
        CloseFrame, SignalingSocketItem, SignalingSocketMessage,
    };
    use opentalk_types_common::modules::module_id;
    use opentalk_types_signaling::ParticipantId;
    use serde_json::{json, value::to_raw_value};
    use tokio::sync::watch;

    use crate::{
        message_router::{
            self, MessageEnvelope, MessageRouter, SignalingMessage, WS_CLOSE_ABNORMAL,
        },
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
            .send(Ok(SignalingSocketItem {
                message: SignalingSocketMessage::Close(Some(CloseFrame {
                    code: WS_CLOSE_ABNORMAL,
                    reason: "this is a test".to_string(),
                })),
                done: None,
            }))
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
            transaction_id: None,
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
