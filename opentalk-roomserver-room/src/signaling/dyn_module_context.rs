// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::cell::RefCell;

use futures::stream::FuturesUnordered;
use opentalk_roomserver_signaling::{
    event_origin::EventOrigin, loopback::LoopbackFuture, module_context::ModuleContext,
    participant_state::Participants, room_info::RoomInfo, signaling_module::SignalingModule,
};
use opentalk_roomserver_types::{
    connection_id::ConnectionId, room_kind::RoomKind, shared_raw_json::SharedRawJson,
};
use opentalk_types_common::rooms::RoomId;

use crate::message_router::MessageRouter;

/// Contains the state of the [`RoomTask`](super::super::task::RoomTask) that is accessible to all [`SignalingModule`]s
pub struct DynModuleContext<'ctx> {
    pub room_id: RoomId,
    pub room: RoomKind,
    pub event_origin: EventOrigin,
    pub room_info: &'ctx mut RoomInfo,
    pub message_router: &'ctx mut MessageRouter,
    pub participants: &'ctx mut Participants,
    loopback_futures: &'ctx FuturesUnordered<LoopbackFuture>,
}

impl<'ctx> DynModuleContext<'ctx> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        room_id: RoomId,
        room: RoomKind,
        event_origin: EventOrigin,
        room_info: &'ctx mut RoomInfo,
        message_router: &'ctx mut MessageRouter,
        participants: &'ctx mut Participants,
        loopback_futures: &'ctx FuturesUnordered<LoopbackFuture>,
    ) -> Self {
        Self {
            room_id,
            room,
            event_origin,
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
            room: self.room,
            event_origin: self.event_origin,
            room_info: self.room_info,
            message_router: self.message_router,
            participants: self.participants,
            loopback_futures: self.loopback_futures,
        }
    }

    pub(crate) fn into_typed_context<M: SignalingModule>(
        self,
        messages: &'ctx mut RefCell<Vec<(ConnectionId, SharedRawJson)>>,
    ) -> ModuleContext<'ctx, M> {
        ModuleContext::new(
            self.room_id,
            self.room,
            self.event_origin,
            self.room_info,
            messages,
            self.participants,
            self.loopback_futures,
        )
    }
}
