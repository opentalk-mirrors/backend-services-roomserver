// SPDX-License-Identifier: EUPL-1.2
//
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use serde::{Deserialize, Serialize};

use crate::breakout::breakout_id::BreakoutId;

pub mod breakout_config;
pub mod breakout_id;
pub mod command;
pub mod event;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakoutRoom {
    pub id: BreakoutId,

    pub name: String,
}
