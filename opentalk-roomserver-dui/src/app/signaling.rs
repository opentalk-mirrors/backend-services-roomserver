// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use egui::RichText;
use tokio::sync::mpsc::{error::TryRecvError, UnboundedReceiver, UnboundedSender};

use crate::{
    app::{
        error::RunnerGoneError,
        event_widget::EventWidget,
        json_edit::json_editor,
        shortcuts::{CLEAR_SHORTCUT, DISCONNECT_SHORTCUT, SEND_SHORTCUT},
        TransitionToView,
    },
    client::{RunnerCommand, RunnerEvent, SignalingState},
    settings::DuiSettings,
};

#[derive(Debug)]
pub struct SignalingView {
    messages: Vec<EventWidget>,

    edit_message: String,
}

impl SignalingView {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            edit_message: String::new(),
        }
    }
    pub fn menu_ui(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        command_tx: &UnboundedSender<RunnerCommand>,
        signaling_state: SignalingState,
    ) -> Result<Option<TransitionToView>, RunnerGoneError> {
        let button =
            egui::Button::new("Clear Messages").shortcut_text(ctx.format_shortcut(&CLEAR_SHORTCUT));
        if ui.add(button).clicked() {
            self.clear_messages(ctx);
        }

        match signaling_state {
            SignalingState::Connected => {
                let button = egui::Button::new("Disconnect")
                    .shortcut_text(ctx.format_shortcut(&DISCONNECT_SHORTCUT));
                if ui.add(button).clicked()
                    || ctx.input_mut(|i| i.consume_shortcut(&DISCONNECT_SHORTCUT))
                {
                    command_tx.send(RunnerCommand::Close)?;
                }
            }
            SignalingState::Disconnect => {
                let button = egui::Button::new("Connect")
                    .shortcut_text(ctx.format_shortcut(&DISCONNECT_SHORTCUT));
                if ui.add(button).clicked() {
                    return Ok(Some(TransitionToView::ConnectionConfig));
                }
                if ctx.input_mut(|i| i.consume_shortcut(&DISCONNECT_SHORTCUT)) {
                    return Ok(Some(TransitionToView::ConnectionConfig));
                }
            }
        }

        if ctx.input_mut(|i| i.consume_shortcut(&CLEAR_SHORTCUT)) {
            self.clear_messages(ctx);
        }

        Ok(None)
    }

    fn clear_messages(&mut self, ctx: &egui::Context) {
        self.messages.clear();
        ctx.request_repaint();
    }

    pub fn center_ui(
        &mut self,
        ui: &mut egui::Ui,
        event_rx: &mut UnboundedReceiver<RunnerEvent>,
        settings: &DuiSettings,
    ) -> Option<TransitionToView> {
        if let Err(_e) = self.receive_runner_events(event_rx) {
            return Some(TransitionToView::Error {
                message: RichText::new("Failed to get new messages from RoomServer task."),
            });
        }

        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for msg in &self.messages {
                    msg.ui(ui, &settings.event_widget_layout);
                }
            });

        None
    }

    fn receive_runner_events(
        &mut self,
        event_rx: &mut UnboundedReceiver<RunnerEvent>,
    ) -> Result<(), RunnerGoneError> {
        loop {
            match event_rx.try_recv() {
                Ok(msg) => self.messages.push(msg.into()),
                Err(TryRecvError::Empty) => return Ok(()),
                Err(TryRecvError::Disconnected) => {
                    log::error!("Failed to receive runner event, channel closed.");
                    return Err(RunnerGoneError);
                }
            }
        }
    }

    fn send_websocket_message(
        &mut self,
        command_tx: &UnboundedSender<RunnerCommand>,
    ) -> Result<(), RunnerGoneError> {
        let mut message = String::new();
        std::mem::swap(&mut message, &mut self.edit_message);
        command_tx.send(RunnerCommand::Send { message })?;

        Ok(())
    }

    pub(crate) fn bottom_ui(
        &mut self,
        ui: &mut egui::Ui,
        command_tx: &UnboundedSender<RunnerCommand>,
        signaling_state: SignalingState,
    ) -> Result<(), RunnerGoneError> {
        ui.horizontal(|ui| {
            let button =
                egui::Button::new("Send").shortcut_text(ui.ctx().format_shortcut(&SEND_SHORTCUT));
            let button_res = ui.add_enabled(signaling_state.is_connected(), button);
            json_editor(ui, &mut self.edit_message);

            if button_res.clicked() || ui.input_mut(|i| i.consume_shortcut(&SEND_SHORTCUT)) {
                self.send_websocket_message(command_tx)
            } else {
                Ok(())
            }
        })
        .inner
    }
}

impl Default for SignalingView {
    fn default() -> Self {
        Self::new()
    }
}
