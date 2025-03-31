// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::HashSet,
    future::Future,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use anyhow::Context;
use opentalk_roomserver_types::{
    error::{self, SignalingError},
    signaling::SignalingEvent,
};
use opentalk_types_common::{modules::ModuleId, rooms::RoomId};
use opentalk_types_signaling::ParticipantId;
use serde::Serialize;
use serde_json::value::RawValue;

use super::{signaling_module::FatalError, SignalingModule};
use crate::room::{
    message_router::MessageRouter,
    task::{LoopbackFutures, LoopbackMessage, RoomInfo},
};

/// Contains the state of the [`RoomTask`](super::super::task::RoomTask) that is accessible to all [`SignalingModule`]s
pub struct DynModuleContext<'ctx> {
    pub room_id: RoomId,
    pub participant_id: ParticipantId,
    pub(crate) room_info: &'ctx mut RoomInfo,
    message_router: &'ctx mut MessageRouter,
    pub participants: &'ctx mut HashSet<ParticipantId>,
    loopback_futures: &'ctx LoopbackFutures,
}

impl<'ctx> DynModuleContext<'ctx> {
    pub(crate) fn new(
        room_id: RoomId,
        participant_id: ParticipantId,
        room_info: &'ctx mut RoomInfo,
        message_router: &'ctx mut MessageRouter,
        participants: &'ctx mut HashSet<ParticipantId>,
        loopback_futures: &'ctx LoopbackFutures,
    ) -> Self {
        Self {
            room_id,
            participant_id,
            room_info,
            message_router,
            participants,
            loopback_futures,
        }
    }

    pub fn room_info(&self) -> &RoomInfo {
        self.room_info
    }

    /// Create a owned copy of self with a narrower lifetime
    pub(crate) fn reborrow(&mut self) -> DynModuleContext<'_> {
        DynModuleContext {
            room_id: self.room_id,
            participant_id: self.participant_id,
            room_info: self.room_info,
            message_router: self.message_router,
            participants: self.participants,
            loopback_futures: self.loopback_futures,
        }
    }

    /// Send a websocket message to the given participant
    ///
    /// # Errors
    ///
    /// Returns a [`FatalError`] when the content fails to serialize
    pub(crate) async fn send_ws_message(
        &mut self,
        participant_id: ParticipantId,
        namespace: ModuleId,
        content: impl Serialize,
    ) -> Result<(), FatalError> {
        let content = serde_json::value::to_raw_value(&content)
            .with_context(|| format!("Failed to serialize message for namespace '{namespace}'"))
            .map_err(FatalError)?;

        self.message_router
            .send_event(participant_id, SignalingEvent { namespace, content })
            .await;

        Ok(())
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

    /// Send a websocket error message of type [`SignalingError`] to the associated participant
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
                self.participant_id,
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

/// Contains the room state as [`DynModuleContext`] and provides an interface to send websocket messages.
///
/// Can be dereferenced to access the state in the inner [`DynModuleContext`]
pub struct ModuleContext<'ctx, M>
where
    M: SignalingModule,
{
    inner: DynModuleContext<'ctx>,
    m: PhantomData<fn() -> M>,
}

// Allows accessing the fields of `inner` without having to expose the field
impl<'ctx, M> Deref for ModuleContext<'ctx, M>
where
    M: SignalingModule,
{
    type Target = DynModuleContext<'ctx>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// Allows accessing the fields of `inner` as mutable without having to expose the field
impl<M> DerefMut for ModuleContext<'_, M>
where
    M: SignalingModule,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'ctx, M> ModuleContext<'ctx, M>
where
    M: SignalingModule,
{
    pub(super) fn new(ctx: DynModuleContext<'ctx>) -> Self {
        Self {
            inner: ctx,
            m: PhantomData,
        }
    }

    /// Send a websocket message of type [`SignalingModule::Outgoing`] to the given `participant_id`
    ///
    /// The message is always scoped to the [`SignalingModule::NAMESPACE`]
    ///
    /// # Errors
    ///
    /// Returns `Err` when the [`SignalingModule::Outgoing`] type failed to be serialized.
    pub async fn send_ws_message(
        &mut self,
        participant_id: ParticipantId,
        msg: M::Outgoing,
    ) -> Result<(), FatalError> {
        self.message_router
            .send_event(
                participant_id,
                SignalingEvent {
                    namespace: M::NAMESPACE,
                    content: serde_json::value::to_raw_value(&msg)
                        .context("Failed to serialize internal websocket payload type")
                        .map_err(FatalError)?,
                },
            )
            .await;

        Ok(())
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

        let future = Box::pin(async move {
            Some(LoopbackMessage {
                namespace: M::NAMESPACE,
                participant_id,
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
        let join_handle = tokio::task::spawn_blocking(blocking_function);

        let future = Box::pin(async move {
            let Ok(value) = join_handle.await else {
                log::error!("module {} panicked in loopback task", M::NAMESPACE);
                return None;
            };

            Some(LoopbackMessage {
                namespace: M::NAMESPACE,
                participant_id,
                value: Box::new(value),
            })
        });

        self.loopback_futures.push(future);
    }
}
