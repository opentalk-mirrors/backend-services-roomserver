// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_types::breakout_id::BreakoutId;
use opentalk_types_common::modules::{ModuleId, module_id};
use serde::{Deserialize, Serialize};

pub mod breakout_config;
pub mod command;
pub mod event;
pub mod module_data;

pub const NAMESPACE: ModuleId = module_id!("breakout");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakoutRoom {
    pub id: BreakoutId,

    pub name: String,
}
