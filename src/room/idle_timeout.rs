// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{pin::Pin, time::Duration};

use tokio::time::Sleep;

/// Helper type to start and stop the idle timeout of the [`super::task::RoomTask`]
pub(super) struct IdleTimeout {
    timeout: Option<Pin<Box<Sleep>>>,
}

impl IdleTimeout {
    /// Creates a new idle timeout
    pub(super) fn start_new(secs: u64) -> Self {
        Self {
            timeout: Some(Box::pin(tokio::time::sleep(Duration::from_secs(secs)))),
        }
    }

    /// Starts a new timeout
    ///
    /// Discards the current timeout if one was set
    pub(super) fn start(&mut self, secs: u64) {
        self.timeout = Some(Box::pin(tokio::time::sleep(Duration::from_secs(secs))))
    }

    /// Refreshes the timeout
    ///
    /// Does nothing when no timeout is currently set
    pub(super) fn refresh(&mut self, secs: u64) {
        if self.timeout.is_some() {
            self.start(secs);
        }
    }

    /// Stops the current timeout
    #[allow(dead_code)] //TODO: remove when used
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
