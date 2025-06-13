// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{net::IpAddr, path::Path};

use eframe::CreationContext;
use egui::ThemePreference;
use opentalk_roomserver_common::settings::{Settings, SettingsFile};
use opentalk_roomserver_types::{
    client_parameters::ClientParameters, room_parameters::RoomParameters,
};
use opentalk_types_common::rooms::RoomId;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    app::event_widget::EventWidgetLayout,
    settings::room::{default_client_parameters, default_room_parameters},
};

mod livekit;
mod message_history;
pub mod room;

pub use livekit::LiveKitSettings;
pub use message_history::{HistoryEntry, MessageHistory};

const SETTINGS_KEY: &str = "settings";

/// Settings for the application.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DuiSettings {
    /// True if no settings where stored before.
    ///
    /// In case no config file or persistent settings where found, the default settings will be used.
    #[serde(skip)]
    is_default: bool,

    /// Theme of the application.
    #[serde(default)]
    pub theme: DuiTheme,

    /// URL of the room server.
    pub roomserver_url: Url,

    /// API token for the room server.
    #[serde(default)]
    pub roomserver_api_token: String,

    /// Layout of the event widget.
    #[serde(default)]
    pub event_widget_layout: EventWidgetLayout,

    /// Message history
    ///
    /// Every new message that is sent will be recorded here.
    #[serde(default)]
    pub history: MessageHistory,

    #[serde(default)]
    pub room_ids: Vec<(String, RoomId)>,
    #[serde(default)]
    pub selected_room_id: usize,

    #[serde(default)]
    pub room_parameters: Vec<(String, RoomParameters)>,
    #[serde(default)]
    pub selected_room_parameters: usize,

    #[serde(default)]
    pub client_parameters: Vec<(String, ClientParameters)>,
    #[serde(default)]
    pub selected_client_parameters: usize,

    #[serde(default)]
    pub delete_mode: bool,

    #[serde(default)]
    pub livekit: LiveKitSettings,
}

impl Default for DuiSettings {
    fn default() -> Self {
        Self {
            theme: DuiTheme::default(),
            roomserver_url: Url::parse("http://localhost:11333").expect("Static URL must be valid"),
            roomserver_api_token: String::new(),
            event_widget_layout: EventWidgetLayout::new(),
            is_default: true,
            history: MessageHistory::default(),

            room_ids: [
                ("Room-1".to_string(), RoomId::from_u128(1)),
                ("Room-2".to_string(), RoomId::from_u128(2)),
                ("Room-3".to_string(), RoomId::from_u128(3)),
            ]
            .to_vec(),
            selected_room_id: 0,

            room_parameters: [("Default".to_string(), default_room_parameters())].to_vec(),
            selected_room_parameters: 0,

            client_parameters: [("Default".to_string(), default_client_parameters())].to_vec(),
            selected_client_parameters: 0,

            delete_mode: false,
            livekit: LiveKitSettings::default(),
        }
    }
}

impl DuiSettings {
    pub fn save(&self, storage: &mut dyn eframe::Storage) {
        storage.set_string(
            SETTINGS_KEY,
            serde_json::to_string(&self).expect("Settings are serializable"),
        );
    }

    pub fn load(
        cc: &CreationContext<'_>,
        roomserver_config: Option<&Path>,
    ) -> Result<DuiSettings, anyhow::Error> {
        let mut settings = if let Some(raw_settings) = cc
            .storage
            .and_then(|storage| storage.get_string(SETTINGS_KEY))
        {
            log::info!("Load settings from persistent storage");
            let mut settings = serde_json::from_str::<DuiSettings>(&raw_settings)?;
            settings.is_default = false;
            log::trace!("loaded settings: {settings:#?}");
            settings
        } else {
            log::info!("No persistent storage");
            DuiSettings::default()
        };
        cc.egui_ctx.set_theme(settings.theme);

        if let Some(config) = roomserver_config {
            log::debug!("Loading Roomserver Configuration");
            let roomserver_settings: Settings = SettingsFile::load_from_path(config)?.into();

            let roomserver_url = build_url(
                roomserver_settings.http.address,
                roomserver_settings.http.port,
            )?;
            let roomserver_api_token = roomserver_settings.http.api_token;

            log::info!("Overwrite roomserver URL and API token");
            settings.roomserver_api_token = roomserver_api_token;
            settings.roomserver_url = roomserver_url;

            settings.is_default = false;
        }
        Ok(settings)
    }

    /// True if no settings where stored before.
    ///
    /// In case no config file or persistent settings where found, the default settings will be used.
    pub fn is_default(&self) -> bool {
        self.is_default
    }

    pub fn mark_custom(&mut self) {
        self.is_default = false;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DuiTheme {
    #[default]
    System,
    Dark,
    Light,
}

impl From<ThemePreference> for DuiTheme {
    fn from(value: ThemePreference) -> Self {
        match value {
            egui::ThemePreference::Dark => DuiTheme::Dark,
            egui::ThemePreference::Light => DuiTheme::Light,
            egui::ThemePreference::System => DuiTheme::System,
        }
    }
}

impl From<DuiTheme> for ThemePreference {
    fn from(value: DuiTheme) -> Self {
        match value {
            DuiTheme::Dark => egui::ThemePreference::Dark,
            DuiTheme::Light => egui::ThemePreference::Light,
            DuiTheme::System => egui::ThemePreference::System,
        }
    }
}

fn build_url(host: IpAddr, port: u16) -> anyhow::Result<Url> {
    let url = if host.is_ipv6() {
        format!("http://[{host}]:{port}")
    } else {
        format!("http://{host}:{port}")
    };
    let url = url.parse()?;
    Ok(url)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_deserialize_with_missing_optional_fields() {
        let json_data = json!({
            "roomserver_url": "http://example.com"
        });

        let settings: DuiSettings =
            serde_json::from_value(json_data).expect("Deserialization failed");
        assert_eq!(settings.theme, DuiTheme::System);
        assert_eq!(settings.roomserver_url.as_str(), "http://example.com/");
        assert_eq!(settings.roomserver_api_token, "");
    }
}
