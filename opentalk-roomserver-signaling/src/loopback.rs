// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{any::Any, future::Future, pin::Pin};

use opentalk_roomserver_types::breakout::breakout_id::BreakoutId;
use opentalk_types_common::modules::ModuleId;

use crate::event_origin::EventOrigin;

pub type LoopbackFuture = Pin<Box<dyn Future<Output = Option<LoopbackMessage>> + Send + Sync>>;

pub struct LoopbackMessage {
    pub namespace: ModuleId,
    pub origin: EventOrigin,
    pub breakout_room: Option<BreakoutId>,
    pub value: Box<dyn Any + Send + 'static>,
}
