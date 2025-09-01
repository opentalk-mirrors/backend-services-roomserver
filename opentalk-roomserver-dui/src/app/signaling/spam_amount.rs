// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use egui::Widget;
use opentalk_roomserver_client::api::command::{SignalingCommand, SignalingModuleCommand};
use opentalk_roomserver_types_chat::{
    Scope,
    command::{ChatCommand, SendMessage},
};

use super::plugin::Received;
use crate::app::{
    shortcuts::TOGGLE_SPAM_AMOUNT_WINDOW_SHORTCUT, signaling::plugin::SignalingPlugin,
};

/// Plugin that allows to spam an amount of chat messages.
#[derive(Debug)]
pub struct SpamAmountPlugin {
    message_size: usize,
    message_count: usize,
    running: bool,
    message: String,
}

impl SignalingPlugin for SpamAmountPlugin {
    fn name(&self) -> &str {
        "Spam Amount"
    }

    fn handle_events(
        &mut self,
        _settings: &mut crate::settings::DuiSettings,
        _received: &[Received],
    ) -> Vec<String> {
        Vec::new()
    }

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _settings: &mut crate::settings::DuiSettings,
    ) -> Vec<String> {
        let mut messages = Vec::new();
        ui.vertical(|ui| {
            self.message_count_ui(ui);
            self.message_size_ui(ui);
            if let Some(msg) = self.command_ui(ui) {
                messages.push(msg);
            }
        });
        messages
    }

    fn shortcut(&self) -> Option<&egui::KeyboardShortcut> {
        Some(&TOGGLE_SPAM_AMOUNT_WINDOW_SHORTCUT)
    }
}

impl SpamAmountPlugin {
    pub fn new() -> Self {
        let message_size = 1024;
        Self {
            message_size,
            message_count: 10,
            running: false,
            message: build_chat_message(message_size),
        }
    }

    fn message_count_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Count");
        egui::DragValue::new(&mut self.message_count).ui(ui);
    }

    fn message_size_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Size (in bytes + chat message overhead)");
        if egui::DragValue::new(&mut self.message_size)
            .ui(ui)
            .changed()
        {
            self.message = build_chat_message(self.message_size);
        }
    }

    fn command_ui(&mut self, ui: &mut egui::Ui) -> Option<String> {
        ui.horizontal(|ui| {
            if !self.running && ui.button("Start").clicked() {
                self.running = true;
            } else if ui.button("Stop").clicked() || self.message_count == 0 {
                self.running = false;
            } else if self.running {
                self.message_count -= 1;
                log::trace!("request repaint: message count decreased");
                ui.ctx().request_repaint();
                return Some(self.message.clone());
            }
            None
        })
        .inner
    }
}

fn build_chat_message(message_size: usize) -> String {
    let content: String = std::iter::repeat_n('~', message_size).collect();
    serde_json::to_string(&SignalingCommand::from(SignalingModuleCommand::Chat(
        ChatCommand::SendMessage(SendMessage {
            content,
            scope: Scope::Global,
        }),
    )))
    .expect("ChatCommand must be serializable")
}
