// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use egui::{Color32, RichText};
use egui_json_tree::{JsonTree, JsonTreeStyle, ToggleButtonsState};

use crate::client::{RunnerEvent, RunnerEventType};

mod event_widget_layout;

pub use event_widget_layout::{EventWidgetLayout, Expand};

#[derive(Debug)]
pub(crate) struct EventWidget {
    event: RunnerEvent,
    json: Option<serde_json::Value>,
    timestamp: String,
}

impl EventWidget {
    pub fn new(event: RunnerEvent) -> Self {
        let json = match &event.event_type {
            RunnerEventType::SendSuccess { message } | RunnerEventType::Received { message } => {
                let value = serde_json::from_str(message);
                value.ok()
            }
            _ => None,
        };
        let timestamp = event.timestamp.format("%T %3f").to_string();
        Self {
            event,
            json,
            timestamp,
        }
    }

    pub(crate) fn ui(&self, ui: &mut egui::Ui, layout: &event_widget_layout::EventWidgetLayout) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

            ui.label(RichText::new(&self.timestamp).color(Color32::GRAY));

            match &self.event.event_type {
                RunnerEventType::Disconnected => {
                    ui.label(RichText::new("Disconnected").color(Color32::RED));
                }
                RunnerEventType::Connected => {
                    ui.label(RichText::new("Connected").color(Color32::GREEN));
                }
                RunnerEventType::Received { message } => {
                    ui.label(
                        RichText::new("⬇")
                            .color(Color32::WHITE)
                            .background_color(Color32::DARK_BLUE),
                    );
                    self.message_ui(ui, message, layout);
                }
                RunnerEventType::ReceiveError { error } => {
                    ui.label(error.to_string());
                }
                RunnerEventType::SendSuccess { message } => {
                    ui.label(
                        RichText::new("⬆")
                            .color(Color32::WHITE)
                            .background_color(Color32::DARK_GREEN),
                    );
                    self.message_ui(ui, message, layout);
                }
                RunnerEventType::SendError { error } => {
                    ui.label(error.to_string());
                }
            }
        });
    }

    fn message_ui(
        &self,
        ui: &mut egui::Ui,
        fallback: &str,
        layout: &event_widget_layout::EventWidgetLayout,
    ) {
        if let Some(json) = &self.json {
            let mut reset_level = false;
            if ui.button("Reset").clicked() {
                reset_level = true;
            }
            let res = JsonTree::new((&self.timestamp, fallback), json)
                .style(
                    JsonTreeStyle::new().toggle_buttons_state(ToggleButtonsState::VisibleEnabled),
                )
                .default_expand(layout.expanded.into())
                .show(ui);
            if reset_level {
                res.reset_expanded(ui);
            }
        } else {
            ui.code(fallback);
        }
    }
}

impl From<RunnerEvent> for EventWidget {
    fn from(value: RunnerEvent) -> Self {
        Self::new(value)
    }
}
