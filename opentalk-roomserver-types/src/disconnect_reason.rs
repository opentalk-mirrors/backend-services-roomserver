// SPDX-License-Identifier: EUPL-1.2
//
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisconnectReason {
    /// The participant left the conference
    Leave,
    /// The connection was interrupted
    ConnectionLost,
    /// The participant was kicked by a moderator
    Kicked,
    /// The participant was removed due to an internal error
    InternalError,
}
