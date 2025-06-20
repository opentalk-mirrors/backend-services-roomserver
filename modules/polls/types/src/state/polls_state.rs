// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Frontend data for `polls` namespace

use std::time::Duration;

use chrono::Utc;
use opentalk_types_common::time::Timestamp;
use serde::{Deserialize, Serialize};

use crate::{Choice, PollId};

/// The state of the `polls` module.
///
/// This struct is sent to the participant in the `join_success` message
/// when they join successfully to the meeting.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PollsState {
    /// The id of the poll
    pub id: PollId,

    /// The description of the poll topic
    pub topic: String,

    /// True if the poll is live
    pub live: bool,

    /// True if the poll accepts multiple choices
    pub multiple_choice: bool,

    /// Choices of the poll
    pub choices: Vec<Choice>,

    /// The time when the poll started
    pub started: Timestamp,

    /// The duration of the poll
    #[serde(with = "opentalk_types_common::utils::duration_seconds")]
    pub duration: Duration,
}

impl opentalk_types_signaling::SignalingModuleFrontendData for PollsState {
    const NAMESPACE: Option<opentalk_types_common::modules::ModuleId> =
        Some(crate::POLLS_MODULE_ID);
}

impl PollsState {
    /// Get the remaining duration of the poll
    pub fn remaining(&self) -> Option<Duration> {
        let duration = chrono::Duration::from_std(self.duration)
            .expect("duration as secs should never be larger than i64::MAX");

        let expire = (*self.started) + duration;
        let now = Utc::now();

        // difference will be negative duration if expired.
        // Conversion to std duration will fail -> returning None
        (expire - now).to_std().ok()
    }

    /// Is the poll expired
    pub fn is_expired(&self) -> bool {
        self.remaining().is_none()
    }
}
