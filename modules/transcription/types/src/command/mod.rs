// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `transcription` namespace

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use serde::{Deserialize, Serialize};

use crate::{event::TranscriptionEvent, service::event::TranscriptionServiceEvent};

/// Commands for the `transcription` namespace
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum TranscriptionCommand {
    Start {
        /// Optional language hint for the transcription service, e.g. "en" or "de".
        #[serde(skip_serializing_if = "Option::is_none")]
        language: Option<String>,
    },

    Stop,

    TranscriptionServiceEvent(TranscriptionServiceEvent),
}

impl CreateReplica<TranscriptionEvent> for TranscriptionCommand {
    fn replicate(&self) -> Option<TranscriptionEvent> {
        None
    }
}
