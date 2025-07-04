// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types_timer::{TimerConfig, event::Stopped};
use tokio::sync::oneshot::Sender;

/// A timer
///
/// Stores information about a running timer
#[derive(Debug)]
pub struct Timer {
    pub config: TimerConfig,
    /// The sender used to cancel the timer
    pub tx_cancel: Option<Sender<Stopped>>,
}
