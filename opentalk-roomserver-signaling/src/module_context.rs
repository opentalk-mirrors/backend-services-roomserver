// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{future::Future, marker::PhantomData};

use anyhow::Context as _;
use futures::stream::FuturesUnordered;
use opentalk_roomserver_types::{
    connection_id::ConnectionId,
    error::{self, SignalingError},
    signaling::SignalingEvent,
};
use opentalk_types_common::rooms::RoomId;
use opentalk_types_signaling::ParticipantId;
use serde_json::value::RawValue;

use crate::{
    loopback::{LoopbackFuture, LoopbackMessage},
    participant_state::Participants,
    room_info::RoomInfo,
    signaling_module::{FatalError, SignalingModule},
};

/// Contains the room state and provides an interface to send websocket messages.
pub struct ModuleContext<'ctx, M>
where
    M: SignalingModule,
{
    pub room_id: RoomId,
    pub participant_id: ParticipantId,
    pub connection_id: ConnectionId,
    room_info: &'ctx mut RoomInfo,
    // TODO use `SharedRawJson` and implement functions to send messages to all/subset of participants without re-allocation
    /// The websocket messages that are sent out after the module finished its event handling
    messages: Vec<(ConnectionId, SignalingEvent)>,
    /// Contains all participants including disconnected ones
    pub participants: &'ctx mut Participants,
    loopback_futures: &'ctx FuturesUnordered<LoopbackFuture>,
    m: PhantomData<fn() -> M>,
}

impl<'ctx, M> ModuleContext<'ctx, M>
where
    M: SignalingModule,
{
    pub fn new(
        room_id: RoomId,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        room_info: &'ctx mut RoomInfo,
        participants: &'ctx mut Participants,

        loopback_futures: &'ctx FuturesUnordered<LoopbackFuture>,
    ) -> ModuleContext<'ctx, M> {
        Self {
            room_id,
            participant_id,
            connection_id,
            room_info,
            messages: Vec::new(),
            participants,
            loopback_futures,
            m: PhantomData,
        }
    }

    pub fn room_info(&self) -> &RoomInfo {
        self.room_info
    }

    pub fn into_messages(self) -> Vec<(ConnectionId, SignalingEvent)> {
        self.messages
    }

    /// Send a websocket message of type [`SignalingModule::Outgoing`] to the given `participant_id`
    ///
    /// The message is always scoped to the [`SignalingModule::NAMESPACE`]
    ///
    /// # Errors
    ///
    /// Returns `Err` when the [`SignalingModule::Outgoing`] type failed to be serialized.
    pub fn send_ws_message(
        &mut self,
        participant_id: ParticipantId,
        msg: M::Outgoing,
    ) -> Result<(), FatalError> {
        let message = SignalingEvent {
            namespace: M::NAMESPACE,
            content: serde_json::value::to_raw_value(&msg)
                .context("Failed to serialize internal websocket payload type")
                .map_err(FatalError)?,
        };

        let Some(state) = self.participants.get_connected(&participant_id) else {
            log::error!(
                "Module '{}' attempted to send a websocket message to unknown participant {participant_id}",
                M::NAMESPACE
            );
            return Ok(());
        };

        for (connection_id, ..) in &state.connections {
            self.messages.push((*connection_id, message.clone()));
        }

        Ok(())
    }

    /// Send a websocket error message of type [`SignalingError`] to the associated participant
    ///
    /// The message is always scoped to the [`error::NAMESPACE`]
    pub fn send_ws_error(&mut self, error: SignalingError) {
        let content = match serde_json::value::to_raw_value(&error) {
            Ok(value) => value,
            Err(err) => {
                log::error!("Failed to serialize SignalingError type: {err}");
                RawValue::from_string(r#"{"error": "internal"}"#.into()).unwrap()
            }
        };

        let message = SignalingEvent {
            namespace: error::NAMESPACE,
            content,
        };

        let Some(state) = self.participants.get_connected(&self.participant_id) else {
            log::error!(
                "Module '{}' attempted to send a websocket error message to unknown participant {}",
                M::NAMESPACE,
                self.participant_id,
            );
            return;
        };

        for (connection_id, ..) in &state.connections {
            self.messages.push((*connection_id, message.clone()));
        }
    }

    /// Spawns a new task that completes the given `future` and sends the result
    /// back to the calling module as [`SignalingModule::Loopback`] in the
    /// [`SignalingModule::on_loopback_event`] method.
    ///
    /// The room task will panic if the provided future panics.
    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = M::Loopback> + Send + Sync + 'static,
    {
        let participant_id = self.participant_id;
        let connection_id = self.connection_id;

        let future = Box::pin(async move {
            Some(LoopbackMessage {
                namespace: M::NAMESPACE,
                participant_id,
                connection_id,
                value: Box::new(future.await),
            })
        });

        self.loopback_futures.push(future);
    }

    /// Spawns a blocking function as a asynchronous task and sends the result
    /// back to the calling module as [`SignalingModule::Loopback`] in the
    /// [`SignalingModule::on_loopback_event`] method.
    ///
    /// If the provided function panics, any results will be discarded and the module won't be notified.
    pub fn spawn_blocking<F>(&self, blocking_function: F)
    where
        F: FnOnce() -> M::Loopback + Send + 'static,
    {
        let participant_id = self.participant_id;
        let connection_id = self.connection_id;
        let join_handle = tokio::task::spawn_blocking(blocking_function);

        let future = Box::pin(async move {
            let Ok(value) = join_handle.await else {
                log::error!("module {} panicked in loopback task", M::NAMESPACE);
                return None;
            };

            Some(LoopbackMessage {
                namespace: M::NAMESPACE,
                participant_id,
                connection_id,
                value: Box::new(value),
            })
        });

        self.loopback_futures.push(future);
    }
}
