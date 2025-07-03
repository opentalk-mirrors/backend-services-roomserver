// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use crate::command::kind::Kind;

/// Start a new timer
#[derive(Debug, Serialize, Deserialize)]
pub struct Start {
    /// The timer kind
    #[serde(flatten)]
    pub kind: Kind,
    /// An optional string tag to flag this timer with a custom style
    pub style: Option<String>,
    /// An optional title for the timer
    pub title: Option<String>,
    /// Flag to allow/disallow participants to mark themselves as ready
    #[serde(default)]
    pub enable_ready_check: bool,
}
