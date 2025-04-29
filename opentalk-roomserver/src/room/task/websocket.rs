// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use anyhow::Context;
use opentalk_roomserver_signaling::signaling_module::FatalError;
use opentalk_roomserver_types::{
    connection_id::ConnectionId,
    error::{self, SignalingError},
    signaling::SignalingEvent,
};
use opentalk_roomserver_web_api::v1::signaling::websocket::SignalingSocket;
use opentalk_types_common::modules::ModuleId;
use serde::Serialize;
use serde_json::value::RawValue;

use super::RoomTask;

impl<Socket: SignalingSocket> RoomTask<Socket> {
    /// Send a websocket message to the given list of connections
    ///
    /// # Errors
    ///
    /// Returns a [`FatalError`] when the content fails to serialize
    pub(crate) async fn serialize_and_send(
        &self,
        connections: impl IntoIterator<Item = ConnectionId>,
        namespace: ModuleId,
        content: impl Serialize,
    ) -> Result<(), FatalError> {
        let content = serde_json::value::to_raw_value(&content)
            .with_context(|| format!("Failed to serialize message for namespace '{namespace}'"))
            .map_err(FatalError)?;

        self.message_router
            .send_event(connections, SignalingEvent { namespace, content })
            .await;

        Ok(())
    }

    /// Broadcast a websocket message to all participants
    ///
    /// Returns a [`FatalError`] when the content fails to serialize.
    pub(crate) async fn serialize_and_broadcast(
        &self,
        namespace: ModuleId,
        content: impl Serialize,
    ) -> Result<(), FatalError> {
        let content = serde_json::value::to_raw_value(&content)
            .with_context(|| format!("Failed to serialize message for namespace '{namespace}'"))
            .map_err(FatalError)?;

        self.message_router
            .broadcast_event(SignalingEvent { namespace, content })
            .await;
        Ok(())
    }

    /// Send a websocket error message of type [`SignalingError`] to the associated connection
    ///
    /// The message is always scoped to the [`error::NAMESPACE`]
    pub(crate) async fn send_error(&self, connection_id: ConnectionId, error: SignalingError) {
        let content = match serde_json::value::to_raw_value(&error) {
            Ok(value) => value,
            Err(err) => {
                log::error!("Failed to serialize SignalingError type: {err}");
                RawValue::from_string(r#"{"error": "internal"}"#.into()).unwrap()
            }
        };

        self.message_router
            .send_event(
                [connection_id],
                SignalingEvent {
                    namespace: error::NAMESPACE,
                    content,
                },
            )
            .await;
    }

    /// Send a websocket error message of type [`SignalingError`] to all participants
    ///
    /// The message is always scoped to the [`error::NAMESPACE`]
    pub(crate) async fn broadcast_error(&self, error: SignalingError) {
        let content = match serde_json::value::to_raw_value(&error) {
            Ok(value) => value,
            Err(err) => {
                log::error!("Failed to serialize SignalingError type: {err}");
                RawValue::from_string(r#"{"error": "internal"}"#.into()).unwrap()
            }
        };

        self.message_router
            .broadcast_event(SignalingEvent {
                namespace: error::NAMESPACE,
                content,
            })
            .await;
    }
}
