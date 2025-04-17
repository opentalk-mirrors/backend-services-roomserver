// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{any::Any, future::Future, pin::Pin};

use opentalk_roomserver_types::{breakout_id::BreakoutId, connection_id::ConnectionId};
use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::ParticipantId;

pub type LoopbackFuture = Pin<Box<dyn Future<Output = Option<LoopbackMessage>> + Send + Sync>>;

pub struct LoopbackMessage {
    pub namespace: ModuleId,
    /// TODO: this might need to be optional at some point
    pub participant_id: ParticipantId,
    pub connection_id: ConnectionId,
    pub breakout_room: Option<BreakoutId>,
    pub value: Box<dyn Any + Send + 'static>,
}
