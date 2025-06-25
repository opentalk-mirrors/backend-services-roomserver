// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Frontend data for `polls` namespace

use std::{
    collections::{BTreeSet, HashMap},
    time::Duration,
};

use chrono::Utc;
use opentalk_types_common::time::Timestamp;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot::Sender;

use crate::{Choice, ChoiceId, Item, PollId};

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

/// Contains the state of a poll and a [`Sender`] to cancel it
#[derive(Debug)]
pub struct Poll {
    /// The state of the poll
    pub state: PollsState,

    /// The votes that were cast
    pub voted_choice_ids: HashMap<ParticipantId, BTreeSet<ChoiceId>>,

    /// Cancels the poll
    pub tx_cancel: Sender<StopKind>,
}

impl Poll {
    /// The current result of the poll
    pub fn results(&self) -> Vec<Item> {
        let votes = self.voted_choice_ids.values().flatten();
        let mut results: HashMap<ChoiceId, u32> = self
            .state
            .choices
            .iter()
            .map(|choice| (choice.id, 0))
            .collect();

        for vote in votes {
            *results.entry(*vote).or_insert(0) += 1;
        }

        let mut results = results
            .into_iter()
            .map(|(id, count)| Item { id, count })
            .collect::<Vec<_>>();
        results.sort_by(|a, b| a.id.cmp(&b.id));
        results
    }
}

/// Determines how a poll was stopped
#[derive(Debug)]
pub enum StopKind {
    /// The poll was stopped by a moderator
    ByModerator,
    /// The poll expired
    Expired,
}
