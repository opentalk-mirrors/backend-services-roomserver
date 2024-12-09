// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{pin::Pin, time::Duration};

use tokio::time::Sleep;

/// Helper type to start and stop the idle timeout of the [`super::RoomTask`]
pub(super) struct IdleTimeout {
    timeout: Option<Pin<Box<Sleep>>>,
}

impl IdleTimeout {
    /// Creates a new idle timeout
    pub(super) fn start_new(duration: Duration) -> Self {
        Self {
            timeout: Some(Box::pin(tokio::time::sleep(duration))),
        }
    }

    /// Starts a new timeout
    ///
    /// Discards the current timeout if one was set
    pub(super) fn start(&mut self, duration: Duration) {
        self.timeout = Some(Box::pin(tokio::time::sleep(duration)))
    }

    /// Refreshes the timeout
    ///
    /// Does nothing when no timeout is currently set
    pub(super) fn refresh(&mut self, secs: Duration) {
        if self.timeout.is_some() {
            self.start(secs);
        }
    }

    /// Stops the current timeout
    pub(super) fn stop(&mut self) {
        self.timeout = None;
    }

    /// Returns only when the timeout is reached
    pub(super) async fn has_timed_out(&mut self) {
        if let Some(timeout) = &mut self.timeout {
            timeout.await
        } else {
            std::future::pending().await
        }
    }
}
