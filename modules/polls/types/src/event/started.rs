// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{Choice, PollId};

/// Event signaling to the participant that the poll has started
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Started {
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

    /// Duration of the poll
    #[serde(with = "opentalk_types_common::utils::duration_seconds")]
    pub duration: Duration,
}
