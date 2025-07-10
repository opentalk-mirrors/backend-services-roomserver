// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// LiveKit settings.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

impl opentalk_types_signaling::SignalingModuleFrontendData for LiveKitSettings {
    const NAMESPACE: Option<opentalk_types_common::modules::ModuleId> =
        Some(crate::LIVEKIT_MODULE_ID);
}
