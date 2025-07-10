// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{cell::RefCell, sync::Arc};

use futures::stream::FuturesUnordered;
use opentalk_roomserver_signaling::{
    event_origin::EventOrigin, loopback::LoopbackFuture, module_context::ModuleContext,
    participant_state::Participants, room_info::RoomTaskInfo, signaling_module::SignalingModule,
    storage::StorageProvider,
};
use opentalk_roomserver_types::{
    connection_id::ConnectionId, room_kind::RoomKind, shared_raw_json::SharedRawJson,
};
use opentalk_types_common::{rooms::RoomId, time::Timestamp};

use crate::message_router::MessageRouter;

/// Contains the state of the [`RoomTask`](super::super::task::RoomTask) that is accessible to all [`SignalingModule`]s
pub struct DynModuleContext<'ctx> {
    pub room_id: RoomId,
    pub room: RoomKind,
    pub event_origin: EventOrigin,
    pub room_task_info: &'ctx mut RoomTaskInfo,
    pub message_router: &'ctx mut MessageRouter,
    pub participants: &'ctx mut Participants,
    pub timestamp: Timestamp,
    pub storage: Arc<dyn StorageProvider>,
    loopback_futures: &'ctx mut FuturesUnordered<LoopbackFuture>,
}

impl<'ctx> DynModuleContext<'ctx> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        room_id: RoomId,
        room: RoomKind,
        event_origin: EventOrigin,
        room_task_info: &'ctx mut RoomTaskInfo,
        message_router: &'ctx mut MessageRouter,
        participants: &'ctx mut Participants,
        timestamp: Timestamp,
        storage: Arc<dyn StorageProvider>,
        loopback_futures: &'ctx mut FuturesUnordered<LoopbackFuture>,
    ) -> Self {
        Self {
            room_id,
            room,
            event_origin,
            room_task_info,
            message_router,
            participants,
            timestamp,
            loopback_futures,
            storage,
        }
    }

    /// Create a owned copy of self with a narrower lifetime
    pub(crate) fn reborrow(&mut self) -> DynModuleContext<'_> {
        DynModuleContext {
            room_id: self.room_id,
            room: self.room,
            event_origin: self.event_origin,
            room_task_info: self.room_task_info,
            message_router: self.message_router,
            participants: self.participants,
            timestamp: self.timestamp,
            loopback_futures: self.loopback_futures,
            storage: Arc::clone(&self.storage),
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
            self.room_task_info,
            messages,
            self.participants,
            self.timestamp,
            self.loopback_futures,
            self.storage,
        )
    }
}
