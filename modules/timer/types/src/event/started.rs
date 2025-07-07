// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use crate::TimerConfig;

/// A timer has been started
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Started {
    /// Config of the started timer
    #[serde(flatten)]
    pub config: TimerConfig,
}
