// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! Manage participant connection tasks

use std::collections::{HashMap, HashSet, hash_map::Entry};

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
use opentalk_types_common::{modules::ModuleId, time::Timestamp};
use opentalk_types_signaling::ParticipantId;
use serde::Serialize;
use serde_json::value::RawValue;
use tokio::sync::{mpsc, watch};

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
    pub waiting_room: ScopedRouter,

    pub conference: ScopedRouter,

    /// The internal receiver for [`room_task_command_sender`](MessageRouter::room_task_command_sender) that contains
    /// messages for the room task. Can be read through the [`recv`](MessageRouter::recv) method.
    room_task_command_receiver: mpsc::Receiver<MessageEnvelope<SignalingMessage>>,
}

impl MessageRouter {
    const COMMAND_CHANNEL_BUFFER_SIZE: usize = 128;

    /// Create a new [`MessageRouter`]
    pub fn new(app_state: watch::Receiver<ApplicationState>) -> Self {
        let (command_channel, command_egress) = mpsc::channel(Self::COMMAND_CHANNEL_BUFFER_SIZE);

        Self {
            waiting_room: ScopedRouter::new(command_channel.clone(), app_state.clone()),
            conference: ScopedRouter::new(command_channel, app_state),
            room_task_command_receiver: command_egress,
        }
    }

    /// Upgrade the specified connections from the waiting room to the conference
    pub fn upgrade_connections<'a>(&mut self, connections: impl Iterator<Item = &'a ConnectionId>) {
        Self::move_connections(&mut self.waiting_room, &mut self.conference, connections);
    }

    pub fn move_to_waiting_room<'a>(
        &mut self,
        connections: impl Iterator<Item = &'a ConnectionId>,
    ) {
        Self::move_connections(&mut self.conference, &mut self.waiting_room, connections);
    }

    fn move_connections<'a>(
        from: &mut ScopedRouter,
        to: &mut ScopedRouter,
        connections: impl Iterator<Item = &'a ConnectionId>,
    ) {
        for connection_id in connections {
            if let Some(handle) = from.connections.remove(connection_id) {
                to.connections.insert(*connection_id, handle);
            } else {
                tracing::error!("Trying to move unknown connection {connection_id}");
            }
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
            self.conference.remove_connection(msg.connection_id);
            self.waiting_room.remove_connection(msg.connection_id);
        }

        msg
    }

    pub fn disconnected(&mut self) -> HashSet<(ConnectionId, ParticipantId)> {
        self.conference
            .disconnects
            .drain()
            .chain(self.waiting_room.disconnects.drain())
            .collect()
    }
}

pub struct ScopedRouter {
    /// A collection of active websocket connections
    connections: HashMap<ConnectionId, ConnectionHandle>,

    disconnects: HashMap<ConnectionId, ParticipantId>,

    /// An internal sender that is given to each [`ParticipantConnectionTask`] to communicate with the [`RoomTask`](super::task::RoomTask)
    ///
    /// [`ParticipantConnectionTask`]: participant_connection::ParticipantConnectionTask
    room_task_command_sender: mpsc::Sender<MessageEnvelope<SignalingMessage>>,

    /// The global application state
    app_state: watch::Receiver<ApplicationState>,
}

impl ScopedRouter {
    /// Create a new [`ScopedRouter`]
    pub fn new(
        room_task_command_sender: mpsc::Sender<MessageEnvelope<SignalingMessage>>,
        app_state: watch::Receiver<ApplicationState>,
    ) -> Self {
        Self {
            connections: HashMap::new(),
            disconnects: HashMap::new(),
            room_task_command_sender,
            app_state,
        }
    }

    pub fn add_connection<S: SignalingSocket + 'static>(
        &mut self,
        participant_id: ParticipantId,
        mut websocket: S,
    ) -> Result<ConnectionId, AlreadyConnectedError> {
        let connection_id = ConnectionId::generate();

        let entry = self.connections.entry(connection_id);
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
        self.connections.remove(&connection_id);
    }

    /// Send a [`SignalingEvent`] to a participant
    pub fn send_event(
        &mut self,
        participant_connections: impl IntoIterator<Item = ConnectionId>,
        event: SharedRawJson,
    ) {
        for id in participant_connections {
            let Entry::Occupied(handle) = self.connections.entry(id) else {
                if !self.disconnects.contains_key(&id) {
                    tracing::warn!("Tried to sent message to unknown connection");
                }
                continue;
            };

            if handle.get().send_event(event.clone()).is_err() {
                let handle = handle.remove();
                self.disconnects.insert(id, handle.participant_id());
            }
        }
    }

    /// Send a [`SignalingEvent`] to **all** participants
    pub fn broadcast_event(&mut self, event: SharedRawJson, excluded_connections: &[ConnectionId]) {
        let mut stale_connections = HashSet::new();

        for (connection_id, connection_handle) in &mut self.connections {
            if excluded_connections.contains(connection_id) {
                continue;
            }

            let cloned_event = event.clone();

            if connection_handle.send_event(cloned_event).is_err() {
                stale_connections.insert(*connection_id);
            }
        }

        // send events to all participants and collect stale connections

        // remove all stale connections
        for connection_id in stale_connections {
            if let Some(handle) = self.connections.remove(&connection_id) {
                self.disconnects
                    .insert(connection_id, handle.participant_id());
            }
        }
    }

    /// Send a websocket message to the given list of connections
    ///
    /// # Errors
    ///
    /// Returns a [`FatalError`] when the content fails to serialize
    pub(crate) fn serialize_and_send(
        &mut self,
        connections: impl IntoIterator<Item = ConnectionId>,
        namespace: ModuleId,
        transaction_id: Option<u64>,
        payload: impl Serialize,
    ) -> Result<(), FatalError> {
        let shared_json = Self::serialize_event(namespace, transaction_id, payload)?;
        self.send_event(connections, shared_json);

        Ok(())
    }

    /// Broadcast a websocket message to all participants
    ///
    /// Returns a [`FatalError`] when the content fails to serialize.
    pub(crate) fn serialize_and_broadcast(
        &mut self,
        namespace: ModuleId,
        transaction_id: Option<u64>,
        payload: impl Serialize,
    ) -> Result<(), FatalError> {
        let shared_json = Self::serialize_event(namespace, transaction_id, payload)?;
        self.broadcast_event(shared_json, &[]);

        Ok(())
    }

    fn serialize_event(
        namespace: ModuleId,
        transaction_id: Option<u64>,
        payload: impl Serialize,
    ) -> Result<SharedRawJson, FatalError> {
        let event = SignalingEvent {
            namespace,
            transaction_id,
            timestamp: Timestamp::now(),
            payload,
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
    pub(crate) fn send_error(
        &mut self,
        connection_id: ConnectionId,
        transaction_id: Option<u64>,
        error: SignalingError,
    ) {
        let event = SignalingEvent {
            namespace: error::NAMESPACE,
            transaction_id,
            timestamp: Timestamp::now(),
            payload: error,
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

        self.send_event([connection_id], shared_json);
    }

    /// Send a websocket error message of type [`SignalingError`] to all participants
    ///
    /// The message is always scoped to the [`error::NAMESPACE`]
    pub(crate) fn broadcast_error(&mut self, transaction_id: Option<u64>, error: SignalingError) {
        let event = SignalingEvent {
            namespace: error::NAMESPACE,
            transaction_id,
            timestamp: Timestamp::now(),
            payload: error,
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

        self.broadcast_event(shared_json, &[]);
    }
}

#[cfg(test)]
mod tests {
    use opentalk_roomserver_common::application_state::ApplicationState;
    use opentalk_roomserver_signaling::signaling_event::SignalingEvent;
    use opentalk_roomserver_web_api::v1::signaling::websocket::{
        CloseFrame, SignalingSocketItem, SignalingSocketMessage,
    };
    use opentalk_types_common::{modules::module_id, time::Timestamp};
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

        let connection = router.conference.add_connection(p1_id, p1_socket).unwrap();

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
            namespace: module_id!("echo"),
            transaction_id: None,
            timestamp: Timestamp::now(),
            payload: to_raw_value(&json!({
                "cool": 12,
                "thing": true,
            }))
            .unwrap(),
        };
        let shared_json = serde_json::value::to_raw_value(&event).unwrap().into();

        router.conference.send_event([connection], shared_json);
    }
}
