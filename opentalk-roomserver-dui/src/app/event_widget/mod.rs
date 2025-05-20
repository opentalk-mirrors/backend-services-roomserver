// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use egui::{Color32, RichText};
use egui_json_tree::{JsonTree, JsonTreeStyle, ToggleButtonsState};

use crate::client::{RunnerEvent, RunnerEventType};

mod event_widget_layout;

pub use event_widget_layout::{EventWidgetLayout, Expand};

use super::signaling::filtered_vec::{Filter, Filterable};

#[derive(Debug)]
pub(crate) struct EventWidget {
    event: RunnerEvent,
    json: Option<serde_json::Value>,
    timestamp: String,

    /// Reset the json tree
    reset_expanded: bool,
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
            reset_expanded: false,
        }
    }

    pub(crate) fn control_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(&self.timestamp).color(Color32::GRAY));

            match &self.event.event_type {
                RunnerEventType::Received { .. } => {
                    ui.label(
                        RichText::new("⬇")
                            .color(Color32::WHITE)
                            .background_color(Color32::DARK_BLUE),
                    );
                    if self.json.is_some() && ui.button("Reset").clicked() {
                        self.reset_expanded = true;
                    }
                }
                RunnerEventType::SendSuccess { .. } => {
                    ui.label(
                        RichText::new("⬆")
                            .color(Color32::WHITE)
                            .background_color(Color32::DARK_GREEN),
                    );
                    if self.json.is_some() && ui.button("Reset").clicked() {
                        self.reset_expanded = true;
                    }
                }
                _ => {}
            }
        });
    }

    pub fn content_ui(
        &mut self,
        ui: &mut egui::Ui,
        layout: &event_widget_layout::EventWidgetLayout,
        show_plain: bool,
    ) {
        ui.vertical(|ui| {
            match &self.event.event_type {
                RunnerEventType::Disconnected => {
                    ui.label(RichText::new("Disconnected").color(Color32::RED));
                }
                RunnerEventType::Connected => {
                    ui.label(RichText::new("Connected").color(Color32::GREEN));
                }
                RunnerEventType::Received { message }
                | RunnerEventType::SendSuccess { message } => match &self.json {
                    Some(json) if !show_plain => {
                        let res = JsonTree::new((&self.timestamp, message), json)
                            .style(
                                JsonTreeStyle::new()
                                    .toggle_buttons_state(ToggleButtonsState::VisibleEnabled),
                            )
                            .default_expand(layout.expanded.into())
                            .show(ui);
                        if self.reset_expanded {
                            res.reset_expanded(ui);
                            self.reset_expanded = false;
                        }
                    }
                    _ => {
                        ui.code(message);
                    }
                },
                RunnerEventType::ReceiveError { error } => {
                    ui.label(error.to_string());
                }

                RunnerEventType::SendError { error } => {
                    ui.label(error.to_string());
                }
            }
            // fill the rest of the available space so that it doesn't grow with
            // more content, but already fills everything that is available.
            ui.allocate_space(ui.available_size());
        });
    }
}

impl From<RunnerEvent> for EventWidget {
    fn from(value: RunnerEvent) -> Self {
        Self::new(value)
    }
}

impl Filterable for EventWidget {
    fn apply(&self, filter: &mut Filter) -> bool {
        match &self.event.event_type {
            RunnerEventType::Disconnected => true,
            RunnerEventType::Connected => true,
            RunnerEventType::ReceiveError { .. } => true,
            RunnerEventType::SendError { .. } => true,

            RunnerEventType::SendSuccess { message } | RunnerEventType::Received { message } => {
                filter.apply(message)
            }
        }
    }
}
