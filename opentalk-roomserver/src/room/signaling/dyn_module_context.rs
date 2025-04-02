// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use anyhow::Context;
use futures::stream::FuturesUnordered;
use opentalk_roomserver_signaling::{
    loopback::LoopbackFuture,
    module_context::ModuleContext,
    participant_state::Participants,
    room_info::RoomInfo,
    signaling_module::{FatalError, SignalingModule},
};
use opentalk_roomserver_types::{
    connection_id::ConnectionId,
    error::{self, SignalingError},
    signaling::SignalingEvent,
};
use opentalk_types_common::{modules::ModuleId, rooms::RoomId};
use opentalk_types_signaling::ParticipantId;
use serde::Serialize;
use serde_json::value::RawValue;

use crate::room::message_router::MessageRouter;

/// Contains the state of the [`RoomTask`](super::super::task::RoomTask) that is accessible to all [`SignalingModule`]s
pub struct DynModuleContext<'ctx> {
    pub room_id: RoomId,
    pub participant_id: ParticipantId,
    pub connection_id: ConnectionId,
    pub room_info: &'ctx mut RoomInfo,
    pub message_router: &'ctx mut MessageRouter,
    pub participants: &'ctx mut Participants,
    loopback_futures: &'ctx FuturesUnordered<LoopbackFuture>,
}

impl<'ctx> DynModuleContext<'ctx> {
    pub(crate) fn new(
        room_id: RoomId,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        room_info: &'ctx mut RoomInfo,
        message_router: &'ctx mut MessageRouter,
        participants: &'ctx mut Participants,
        loopback_futures: &'ctx FuturesUnordered<LoopbackFuture>,
    ) -> Self {
        Self {
            room_id,
            participant_id,
            connection_id,
            room_info,
            message_router,
            participants,
            loopback_futures,
        }
    }

    /// Create a owned copy of self with a narrower lifetime
    pub(crate) fn reborrow(&mut self) -> DynModuleContext<'_> {
        DynModuleContext {
            room_id: self.room_id,
            participant_id: self.participant_id,
            connection_id: self.connection_id,
            room_info: self.room_info,
            message_router: self.message_router,
            participants: self.participants,
            loopback_futures: self.loopback_futures,
        }
    }

    /// Send a websocket message to the given list of connections
    ///
    /// # Errors
    ///
    /// Returns a [`FatalError`] when the content fails to serialize
    pub(crate) async fn send_ws_message(
        &mut self,
        connections: impl IntoIterator<Item = ConnectionId>,
        namespace: ModuleId,
        content: impl Serialize,
    ) -> Result<(), FatalError> {
        send_ws_message(self.message_router, connections, namespace, content).await
    }

    /// Broadcast a websocket message to all participants
    ///
    /// Returns a [`FatalError`] when the content fails to serialize.
    pub(crate) async fn broadcast_ws_message(
        &mut self,
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
    pub(crate) async fn send_ws_error(&mut self, error: SignalingError) {
        let content = match serde_json::value::to_raw_value(&error) {
            Ok(value) => value,
            Err(err) => {
                log::error!("Failed to serialize SignalingError type: {err}");
                RawValue::from_string(r#"{"error": "internal"}"#.into()).unwrap()
            }
        };

        self.message_router
            .send_event(
                [self.connection_id],
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
    pub(crate) async fn broadcast_ws_error(&mut self, error: SignalingError) {
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

impl<'ctx, M: SignalingModule> From<DynModuleContext<'ctx>> for ModuleContext<'ctx, M> {
    fn from(ctx: DynModuleContext<'ctx>) -> Self {
        ModuleContext::new(
            ctx.room_id,
            ctx.participant_id,
            ctx.connection_id,
            ctx.room_info,
            ctx.participants,
            ctx.loopback_futures,
        )
    }
}

pub(crate) async fn send_ws_message(
    message_router: &mut MessageRouter,
    connections: impl IntoIterator<Item = ConnectionId>,
    namespace: ModuleId,
    content: impl Serialize,
) -> Result<(), FatalError> {
    let content = serde_json::value::to_raw_value(&content)
        .with_context(|| format!("Failed to serialize message for namespace '{namespace}'"))
        .map_err(FatalError)?;

    message_router
        .send_event(connections, SignalingEvent { namespace, content })
        .await;

    Ok(())
}
