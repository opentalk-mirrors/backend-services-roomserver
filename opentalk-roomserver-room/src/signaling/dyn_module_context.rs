// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{cell::RefCell, collections::HashMap, sync::Arc};

use futures::stream::FuturesUnordered;
use opentalk_roomserver_signaling::{
    banned_participant::BannedParticipant,
    event_origin::EventOrigin,
    loopback::LoopbackFuture,
    module_context::{ModuleContext, ModuleMessage},
    participant_state::Participants,
    room_info::RoomTaskInfo,
    signaling_module::SignalingModule,
    storage::provider::AssetStorageProvider,
    waiting_participant::WaitingParticipant,
};
use opentalk_roomserver_types::room_kind::RoomKind;
use opentalk_types_common::{rooms::RoomId, time::Timestamp};
use opentalk_types_signaling::ParticipantId;

/// Contains the state of the [`RoomTask`](super::super::task::RoomTask) that is accessible to all
/// [`SignalingModule`]s
pub struct DynModuleContext<'ctx> {
    pub room_id: RoomId,
    pub room: RoomKind,
    pub event_origin: EventOrigin,
    pub room_task_info: &'ctx mut RoomTaskInfo,
    pub participants: &'ctx mut Participants,
    pub waiting_participants: &'ctx mut HashMap<ParticipantId, WaitingParticipant>,
    pub banned_participants: &'ctx mut HashMap<ParticipantId, BannedParticipant>,
    pub timestamp: Timestamp,
    pub storage: Arc<dyn AssetStorageProvider>,
    pub messages: &'ctx mut RefCell<Vec<ModuleMessage>>,
    loopback_futures: &'ctx mut FuturesUnordered<LoopbackFuture>,
}

impl<'ctx> DynModuleContext<'ctx> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        room_id: RoomId,
        room: RoomKind,
        event_origin: EventOrigin,
        room_task_info: &'ctx mut RoomTaskInfo,
        participants: &'ctx mut Participants,
        waiting_participants: &'ctx mut HashMap<ParticipantId, WaitingParticipant>,
        banned_participants: &'ctx mut HashMap<ParticipantId, BannedParticipant>,
        timestamp: Timestamp,
        storage: Arc<dyn AssetStorageProvider>,
        messages: &'ctx mut RefCell<Vec<ModuleMessage>>,
        loopback_futures: &'ctx mut FuturesUnordered<LoopbackFuture>,
    ) -> Self {
        Self {
            room_id,
            room,
            event_origin,
            room_task_info,
            participants,
            waiting_participants,
            banned_participants,
            timestamp,
            storage,
            messages,
            loopback_futures,
        }
    }

    /// Create a owned copy of self with a narrower lifetime
    pub(crate) fn reborrow(&mut self) -> DynModuleContext<'_> {
        DynModuleContext {
            room_id: self.room_id,
            room: self.room,
            event_origin: self.event_origin,
            room_task_info: self.room_task_info,
            participants: self.participants,
            waiting_participants: self.waiting_participants,
            banned_participants: self.banned_participants,
            timestamp: self.timestamp,
            storage: Arc::clone(&self.storage),
            messages: self.messages,
            loopback_futures: self.loopback_futures,
        }
    }

    pub(crate) fn into_typed_context<M: SignalingModule>(self) -> ModuleContext<'ctx, M> {
        ModuleContext::new(
            self.room_id,
            self.room,
            self.event_origin,
            self.room_task_info,
            self.messages,
            self.participants,
            self.waiting_participants,
            self.banned_participants,
            self.timestamp,
            self.loopback_futures,
            self.storage,
        )
    }
}
