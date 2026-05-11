// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types::module_settings::SignalingModuleSettings;
use opentalk_service_auth::ApiKey;
use opentalk_types_common::modules::ModuleId;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::TRANSCRIPTION_MODULE_ID;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranscriptionSettings {
    /// The URL of the transcription service
    pub url: Url,
    /// The API key to authenticate with the transcription service
    pub api_key: ApiKey,
}

impl SignalingModuleSettings for TranscriptionSettings {
    const NAMESPACE: ModuleId = TRANSCRIPTION_MODULE_ID;
}
