// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use futures::stream::FuturesUnordered;
use opentalk_roomserver_signaling::{
    loopback::LoopbackFuture, module_context::ModuleContext, participant_state::Participants,
    room_info::RoomInfo, signaling_module::SignalingModule,
};
use opentalk_roomserver_types::connection_id::ConnectionId;
use opentalk_types_common::rooms::RoomId;
use opentalk_types_signaling::ParticipantId;

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

    pub(crate) fn into_typed_context<M: SignalingModule>(self) -> ModuleContext<'ctx, M> {
        ModuleContext::new(
            self.room_id,
            self.participant_id,
            self.connection_id,
            self.room_info,
            self.participants,
            self.loopback_futures,
        )
    }
}
