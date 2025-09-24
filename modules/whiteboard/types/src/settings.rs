// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types::module_settings::SignalingModuleSettings;
use opentalk_types_common::modules::ModuleId;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::WHITEBOARD_MODULE_ID;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhiteboardSettings {
    /// The base URL of the Spacedeck instance.
    pub base_url: Url,

    /// The API key for accessing the Spacedeck instance.
    pub api_key: String,
}

impl SignalingModuleSettings for WhiteboardSettings {
    const NAMESPACE: ModuleId = WHITEBOARD_MODULE_ID;
}
