// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

/// Stop a running timer
#[derive(Debug, Serialize, Deserialize)]
pub struct Stop {
    /// An optional reason for the stop
    pub reason: Option<String>,
}
