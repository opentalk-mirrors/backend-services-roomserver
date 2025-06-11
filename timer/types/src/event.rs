// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling_timer::event::{Started, Stopped, UpdatedReadyStatus};
use serde::{Deserialize, Serialize};

use crate::error::TimerError;

/// Outgoing websocket messages
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimerEvent {
    /// A timer has been started
    Started(Started),
    /// The current timer has been stopped
    Stopped(Stopped),
    /// A participant updated its ready status
    UpdatedReadyStatus(UpdatedReadyStatus),
    /// An error occurred
    Error(TimerError),
}

impl From<Stopped> for TimerEvent {
    fn from(value: Stopped) -> Self {
        Self::Stopped(value)
    }
}

impl From<UpdatedReadyStatus> for TimerEvent {
    fn from(value: UpdatedReadyStatus) -> Self {
        Self::UpdatedReadyStatus(value)
    }
}

impl From<TimerError> for TimerEvent {
    fn from(err: TimerError) -> Self {
        Self::Error(err)
    }
}
