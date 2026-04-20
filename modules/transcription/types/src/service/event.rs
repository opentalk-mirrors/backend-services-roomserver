// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use crate::segment::TranscriptionSegment;

/// Signaling events from the transcription service to the roomserver
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TranscriptionServiceEvent {
    /// Transcription service has been started
    Started,

    /// Transcription service has stopped
    Stopped,

    /// Transcription segment to be sent
    Segment(TranscriptionSegment),
}
