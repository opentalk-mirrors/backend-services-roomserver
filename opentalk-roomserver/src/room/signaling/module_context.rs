// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::HashSet,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use opentalk_roomserver_types::{
    room_parameters::RoomParameters,
    signaling::{SignalingError, SignalingEvent},
};
use opentalk_types_common::rooms::RoomId;
use opentalk_types_signaling::ParticipantId;

use super::SignalingModule;
use crate::room::message_router::MessageRouter;

/// Contains the state of the [`RoomTask`](super::super::task::RoomTask) that is accessible to all [`SignalingModule`]s
pub struct DynModuleContext<'ctx> {
    pub room_id: RoomId,
    pub parameters: &'ctx RoomParameters,
    message_router: &'ctx mut MessageRouter,
    pub participants: &'ctx HashSet<ParticipantId>,
}

impl<'ctx> DynModuleContext<'ctx> {
    pub(crate) fn new(
        room_id: RoomId,
        parameters: &'ctx RoomParameters,
        message_router: &'ctx mut MessageRouter,
        participants: &'ctx HashSet<ParticipantId>,
    ) -> Self {
        Self {
            room_id,
            parameters,
            message_router,
            participants,
        }
    }

    pub(crate) fn reborrow(&mut self) -> DynModuleContext<'_> {
        DynModuleContext {
            room_id: self.room_id,
            parameters: self.parameters,
            message_router: self.message_router,
            participants: self.participants,
        }
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

    /// Send a websocket error message of type [`SignalingError`] to the given `participant_id`
    ///
    /// The message is always scoped to the [`SignalingModule::NAMESPACE`]
    pub async fn send_ws_error(&mut self, participant_id: ParticipantId, error: SignalingError) {
        let Ok(content) = serde_json::to_value(error) else {
            log::error!("failed to serialize error type");
            return;
        };

        self.message_router
            .send_event(
                participant_id,
                SignalingEvent {
                    namespace: M::NAMESPACE.to_string(),
                    content,
                },
            )
            .await;
    }

    /// Send a websocket message of type [`SignalingModule::Outgoing`] to the given `participant_id`
    ///
    /// The message is always scoped to the [`SignalingModule::NAMESPACE`]
    pub async fn send_ws_message(
        &mut self,
        participant_id: ParticipantId,
        msg: M::Outgoing,
    ) -> anyhow::Result<()> {
        self.message_router
            .send_event(
                participant_id,
                SignalingEvent {
                    namespace: M::NAMESPACE.to_string(),
                    content: serde_json::to_value(msg)?,
                },
            )
            .await;

        Ok(())
    }
}
