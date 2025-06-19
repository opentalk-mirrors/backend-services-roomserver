// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

/// LiveKit settings.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
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
