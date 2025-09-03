// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::SignalingModuleFrontendData;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::MEETING_NOTES_MODULE_ID;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeetingNotesSettings {
    /// The base URL of the Etherpad instance.
    pub base_url: Url,

    /// The API key for accessing the Etherpad instance.
    pub api_key: String,
}

impl SignalingModuleFrontendData for MeetingNotesSettings {
    const NAMESPACE: Option<ModuleId> = Some(MEETING_NOTES_MODULE_ID);
}
