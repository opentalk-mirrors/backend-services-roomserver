// SPDX-License-Identifier: EUPL-1.2
//
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use serde::{Deserialize, Serialize};

use crate::breakout::breakout_id::BreakoutId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case", tag = "kind", content = "id")]
pub enum RoomKind {
    Main,
    Breakout(BreakoutId),
}
