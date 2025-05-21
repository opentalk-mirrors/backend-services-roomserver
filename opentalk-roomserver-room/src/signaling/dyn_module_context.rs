// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::cell::RefCell;

use futures::stream::FuturesUnordered;
use opentalk_roomserver_signaling::{
    loopback::LoopbackFuture,
    module_context::ModuleContext,
    participant_state::Participants,
    room_info::RoomInfo,
    signaling_module::{SharedRawJson, SignalingModule},
};
use opentalk_roomserver_types::{breakout_id::BreakoutId, connection_id::ConnectionId};
use opentalk_types_common::rooms::RoomId;
use opentalk_types_signaling::ParticipantId;

use crate::message_router::MessageRouter;

/// Contains the state of the [`RoomTask`](super::super::task::RoomTask) that is accessible to all [`SignalingModule`]s
pub struct DynModuleContext<'ctx> {
    pub room_id: RoomId,
    pub breakout_room: Option<BreakoutId>,
    pub participant_id: ParticipantId,
    pub connection_id: ConnectionId,
    pub room_info: &'ctx mut RoomInfo,
    pub message_router: &'ctx mut MessageRouter,
    pub participants: &'ctx mut Participants,
    loopback_futures: &'ctx FuturesUnordered<LoopbackFuture>,
    transaction_id: Option<u64>,
}

impl<'ctx> DynModuleContext<'ctx> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        room_id: RoomId,
        breakout_room: Option<BreakoutId>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        room_info: &'ctx mut RoomInfo,
        message_router: &'ctx mut MessageRouter,
        participants: &'ctx mut Participants,
        loopback_futures: &'ctx FuturesUnordered<LoopbackFuture>,
        transaction_id: Option<u64>,
    ) -> Self {
        Self {
            room_id,
            breakout_room,
            participant_id,
            connection_id,
            room_info,
            message_router,
            participants,
            loopback_futures,
            transaction_id,
        }
    }

    /// Create a owned copy of self with a narrower lifetime
    pub(crate) fn reborrow(&mut self) -> DynModuleContext<'_> {
        DynModuleContext {
            room_id: self.room_id,
            breakout_room: self.breakout_room,
            participant_id: self.participant_id,
            connection_id: self.connection_id,
            room_info: self.room_info,
            message_router: self.message_router,
            participants: self.participants,
            loopback_futures: self.loopback_futures,
            transaction_id: self.transaction_id,
        }
    }

    pub(crate) fn into_typed_context<M: SignalingModule>(
        self,
        messages: &'ctx mut RefCell<Vec<(ConnectionId, SharedRawJson)>>,
    ) -> ModuleContext<'ctx, M> {
        ModuleContext::new(
            self.room_id,
            self.breakout_room,
            self.participant_id,
            self.connection_id,
            self.room_info,
            messages,
            self.participants,
            self.loopback_futures,
            self.transaction_id,
        )
    }
}
