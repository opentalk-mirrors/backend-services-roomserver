// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::time::Duration;

use crate::settings::{settings_file, signaling_salt::SignalingSalt};

/// The timeout for an empty room
///
/// Should be higher than the lifetime of the signaling token from the token store to ensure that
/// the room doesn't expire before the signaling token does.
pub(crate) const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_mins(1);

#[derive(Debug, Default, Clone)]
pub struct Conference {
    pub signaling_salt: SignalingSalt,

    pub room_idle_timeout: Duration,
}

impl From<settings_file::conference::Conference> for Conference {
    fn from(value: settings_file::conference::Conference) -> Self {
        Self {
            signaling_salt: value.signaling_salt,
            room_idle_timeout: value
                .room_idle_timeout
                .map(Duration::from_secs)
                .unwrap_or(DEFAULT_IDLE_TIMEOUT),
        }
    }
}
