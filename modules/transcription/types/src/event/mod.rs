// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `transcription` namespace

use serde::{Deserialize, Serialize};

mod error;

pub use error::TranscriptionError;

use crate::{
    segment::TranscriptionSegment, service::command::TranscriptionServiceCommand,
    state::TranscriptionStatus,
};

/// Events for the `transcription` namespace
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum TranscriptionEvent {
    Segment(TranscriptionSegment),

    StateUpdated { status: TranscriptionStatus },

    ServiceCommand(TranscriptionServiceCommand),

    Error(TranscriptionError),
}

impl From<TranscriptionError> for TranscriptionEvent {
    fn from(value: TranscriptionError) -> Self {
        Self::Error(value)
    }
}
