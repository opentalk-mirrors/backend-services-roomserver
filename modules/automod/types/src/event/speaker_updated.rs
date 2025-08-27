// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

/// The current speaker state has changed
///
/// This event will ALWAYS notify of a speaker change, even if the speaker is the same participant
/// as before, it MUST be handled as changed.
///
/// Both `history` and `remaining`: If the field is set it will contain the complete new list.
/// If it doesnt exist it must be treated as unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpeakerUpdated {
    /// Speaker field. If [`None`] no speaker is currently selected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speaker: Option<ParticipantId>,

    /// Optional modification of the history.
    ///
    /// If set the frontend MUST replace its history with the given one.
    /// If not set the frontend MUST keep its current history.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<ParticipantId>>,

    /// Optional modification of the remaining participants.
    ///
    /// Remaining participants must be interpreted differently depending on the selection strategy.
    /// E.g. in the playlist moderation `remaining` lists the participants left inside the playlist.
    /// All other strategies will use `remaining` (if at all) to list all participants (if public)
    /// that are eligible to be selected.
    ///
    /// This will only be set when using the `playlist` selection_strategy.
    ///
    /// If set the frontend MUST replace its remaining list with the given one.
    /// If not set the frontend MUST keep its current remaining list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining: Option<Vec<ParticipantId>>,
}
