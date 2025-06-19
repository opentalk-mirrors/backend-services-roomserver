// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::settings::settings_file;

/// LiveKit settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveKitSettings {
    /// The API key for connecting to LiveKit.
    pub api_key: String,

    /// The API secret for connecting to LiveKit.
    pub api_secret: String,

    /// The public url that OpenTalk clients will use for connecting to LiveKit.
    pub public_url: String,

    /// The url that the OpenTalk controller will use for connecting to LiveKit.
    pub service_url: String,
}

impl From<settings_file::livekit::LiveKitSettings> for LiveKitSettings {
    fn from(value: settings_file::livekit::LiveKitSettings) -> Self {
        Self {
            api_key: value.api_key,
            api_secret: value.api_secret,
            public_url: value.public_url,
            service_url: value.service_url,
        }
    }
}
