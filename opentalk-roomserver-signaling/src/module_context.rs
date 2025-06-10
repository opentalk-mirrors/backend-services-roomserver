// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{cell::RefCell, future::Future, marker::PhantomData};

use anyhow::Context as _;
use futures::stream::FuturesUnordered;
use opentalk_roomserver_types::{
    breakout::breakout_id::BreakoutId,
    client_parameters::Role,
    connection_id::ConnectionId,
    error::{self, SignalingError},
    shared_raw_json::SharedRawJson,
    signaling::module_error::FatalError,
};
use opentalk_types_common::rooms::RoomId;
use opentalk_types_signaling::ParticipantId;
use serde_json::value::RawValue;

use crate::{
    event_origin::EventOrigin,
    loopback::{LoopbackFuture, LoopbackMessage},
    participant_state::{ParticipantState, Participants},
    room_info::RoomInfo,
    signaling_event::SignalingEvent,
    signaling_module::SignalingModule,
};

/// Contains the room state and provides an interface to send websocket messages.
pub struct ModuleContext<'ctx, M>
where
    M: SignalingModule,
{
    pub room_id: RoomId,
    pub breakout_room: Option<BreakoutId>,
    pub event_origin: EventOrigin,
    room_info: &'ctx mut RoomInfo,
    /// The websocket messages that are sent out after the module finished its event handling
    messages: &'ctx mut RefCell<Vec<(ConnectionId, SharedRawJson)>>,
    /// Contains all participants including disconnected ones
    pub participants: &'ctx mut Participants,
    loopback_futures: &'ctx FuturesUnordered<LoopbackFuture>,

    m: PhantomData<fn() -> M>,
}

impl<'ctx, M> ModuleContext<'ctx, M>
where
    M: SignalingModule,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        room_id: RoomId,
        breakout_room: Option<BreakoutId>,
        event_origin: EventOrigin,
        room_info: &'ctx mut RoomInfo,
        messages: &'ctx mut RefCell<Vec<(ConnectionId, SharedRawJson)>>,
        participants: &'ctx mut Participants,
        loopback_futures: &'ctx FuturesUnordered<LoopbackFuture>,
    ) -> ModuleContext<'ctx, M> {
        Self {
            room_id,
            breakout_room,
            event_origin,
            room_info,
            messages,
            participants,
            loopback_futures,
            m: PhantomData,
        }
    }

    pub fn reborrow<M2: SignalingModule>(&mut self) -> ModuleContext<'_, M2> {
        ModuleContext {
            room_id: self.room_id,
            breakout_room: self.breakout_room,
            event_origin: self.event_origin,
            room_info: self.room_info,
            messages: self.messages,
            participants: self.participants,
            loopback_futures: self.loopback_futures,
            m: PhantomData,
        }
    }

    pub fn room_info(&self) -> &RoomInfo {
        self.room_info
    }

    /// Send a websocket message of type [`SignalingModule::Outgoing`] to the given `participant_ids`
    ///
    /// The message is always scoped to the [`SignalingModule::NAMESPACE`]
    ///
    /// # Errors
    ///
    /// Returns `Err` when the [`SignalingModule::Outgoing`] type failed to be serialized.
    pub fn send_ws_message(
        &self,
        participant_ids: impl IntoIterator<Item = ParticipantId>,
        msg: M::Outgoing,
    ) -> Result<(), FatalError> {
        let event = SignalingEvent {
            namespace: M::NAMESPACE,
            transaction_id: self.event_origin.transaction_id(),
            content: msg,
        };
        let shared_json: SharedRawJson = serde_json::value::to_raw_value(&event)
            .context("Failed to serialize internal websocket payload type")
            .map_err(FatalError)?
            .into();

        for participant_id in participant_ids {
            let Some(state) = self.participants.connected().get(&participant_id) else {
                log::error!(
                    "Module '{}' attempted to send a websocket message to unknown participant {participant_id}",
                    M::NAMESPACE
                );
                return Ok(());
            };
            let mut messages = self.messages.borrow_mut();

            for (connection_id, ..) in &state.connections {
                messages.push((*connection_id, shared_json.clone()));
            }
        }

        Ok(())
    }

    /// Send a websocket command received from one `source_connection` to all
    /// other connections of the same participant.
    ///
    /// The message is always scoped to the [`SignalingModule::NAMESPACE`]
    ///
    /// # Errors
    ///
    /// Returns [`FatalError`] when the [`SignalingEvent`] type failed to be serialized.
    pub fn send_replica(
        &self,
        sender: ParticipantId,
        source_connection: ConnectionId,
        replication_event: M::Outgoing,
    ) -> Result<(), FatalError> {
        let event = SignalingEvent {
            namespace: M::NAMESPACE,
            transaction_id: self.event_origin.transaction_id(),
            content: replication_event,
        };

        let shared_json: SharedRawJson = serde_json::value::to_raw_value(&event)
            .context("Failed to serialize internal websocket payload type")
            .map_err(FatalError)?
            .into();

        let Some(state) = self.participants.connected().get(&sender) else {
            log::error!(
                "Module '{}' attempted to replicate a command to unknown participant {sender}",
                M::NAMESPACE
            );
            return Ok(());
        };
        let mut messages = self.messages.borrow_mut();

        for connection_id in state.connections.keys().copied() {
            if connection_id != source_connection {
                messages.push((connection_id, shared_json.clone()));
            }
        }

        Ok(())
    }

    /// Invoke an error message of type [`SignalingError`]
    ///
    /// If the event origin is a signaling connection, the error will be sent to the participant.
    ///
    /// The message is always scoped to the [`error::NAMESPACE`]
    pub fn handle_error(&self, error: SignalingError) {
        let participant_id = match self.event_origin {
            EventOrigin::Participant(participant_origin) => participant_origin.id,
            EventOrigin::Internal => {
                log::error!(
                    "Signaling module '{}' returned an error on an event with internal origin: {error:?} ",
                    M::NAMESPACE
                );
                return;
            }
        };

        let event = SignalingEvent {
            namespace: error::NAMESPACE,
            transaction_id: self.event_origin.transaction_id(),
            content: error,
        };

        let shared_json: SharedRawJson = match serde_json::value::to_raw_value(&event) {
            Ok(value) => value.into(),
            Err(err) => {
                log::error!("Failed to serialize SignalingError type: {err}");
                RawValue::from_string(r#"{"error": "internal"}"#.into())
                    .unwrap()
                    .into()
            }
        };

        let Some(state) = self.participants.connected().get(&participant_id) else {
            log::error!(
                "Module '{}' attempted to send a websocket error message to unknown participant {}",
                M::NAMESPACE,
                participant_id,
            );
            return;
        };

        let mut messages = self.messages.borrow_mut();

        for (connection_id, ..) in &state.connections {
            messages.push((*connection_id, shared_json.clone()));
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
        let origin = self.event_origin;
        let breakout_room = self.breakout_room;

        let future = Box::pin(async move {
            Some(LoopbackMessage {
                namespace: M::NAMESPACE,
                origin,
                breakout_room,
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
        let origin = self.event_origin;
        let breakout_room = self.breakout_room;
        let join_handle = tokio::task::spawn_blocking(blocking_function);

        let future = Box::pin(async move {
            let Ok(value) = join_handle.await else {
                log::error!("module {} panicked in loopback task", M::NAMESPACE);
                return None;
            };

            Some(LoopbackMessage {
                namespace: M::NAMESPACE,
                origin,
                breakout_room,
                value: Box::new(value),
            })
        });

        self.loopback_futures.push(future);
    }

    pub fn participant_state(&self, participant_id: ParticipantId) -> Option<&ParticipantState> {
        self.participants.all_unfiltered.get(&participant_id)
    }

    pub fn participant_role(&self, participant_id: ParticipantId) -> Option<Role> {
        self.participant_state(participant_id).map(|p| p.role)
    }

    pub fn is_moderator(&self, participant_id: ParticipantId) -> bool {
        self.participant_role(participant_id)
            .map(|r| r == Role::Moderator)
            .unwrap_or(false)
    }
}
