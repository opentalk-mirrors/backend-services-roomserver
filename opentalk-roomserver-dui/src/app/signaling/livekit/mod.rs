// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use anyhow::Context as _;
use egui::{Color32, RichText, Widget};
use livekit::RoomEvent;
use opentalk_roomserver_client::api::{
    command::{LiveKitCommand, SignalingCommand, SignalingModuleCommand},
    event::{
        Credentials, LiveKitError, LiveKitEvent, LiveKitState, SignalingEvent, SignalingModuleEvent,
    },
};
use opentalk_roomserver_types::{core_event::CoreEvent, join::join_success::JoinSuccess};
use tokio::runtime::Runtime;

use crate::app::{
    shortcuts::TOGGLE_LIVEKIT_WINDOW_SHORTCUT,
    signaling::{
        livekit::{handle::RunnerHandle, runner::LiveKitRunner},
        plugin::{Received, SignalingPlugin},
    },
};

mod handle;
mod runner;

#[derive(Debug)]
pub struct LiveKitPlugin {
    events: Vec<String>,
    credentials: Option<Credentials>,

    handle: RunnerHandle,
}

impl LiveKitPlugin {
    pub fn new(runtime: &Runtime, ctx: egui::Context) -> Self {
        let handle = LiveKitRunner::spawn(runtime, ctx);
        Self {
            events: Vec::new(),
            credentials: None,
            handle,
        }
    }

    fn handle_join_success(&mut self, join: &JoinSuccess) -> anyhow::Result<()> {
        match join.get_module::<LiveKitState>() {
            Ok(Some(state)) => {
                self.events.push(format!(
                    "Livekit state with token: {}",
                    state.credentials.token
                ));
                self.credentials.replace(state.credentials.clone());
                self.handle.connect(state.credentials)?;
            }
            Ok(None) => {
                self.events
                    .push("No LiveKit state found in JoinSuccess".to_string());
            }
            Err(e) => {
                self.events
                    .push(format!("Failed to parse LiveKit state: {}", e));
            }
        }
        Ok(())
    }

    fn handle_livekit_error(&mut self, e: &LiveKitError) -> anyhow::Result<()> {
        self.events.push(format!("LiveKit Error: {:?}", e));
        let _ = self.handle.disconnect();
        Ok(())
    }

    fn handle_runner_event(&mut self, event: RoomEvent) {
        self.events.push(format!("LiveKit Event: {:?}", event));
    }

    fn connection_status_ui(&mut self, ui: &mut egui::Ui) {
        match *self.handle.status_rx.borrow() {
            handle::Status::Disconnected => {
                ui.label(RichText::new("Disconnected").color(Color32::RED))
            }
            handle::Status::Connected => ui.label(RichText::new("Connected").color(Color32::GREEN)),
        };
    }

    fn toggle_connection_ui(
        &mut self,
        messages: &mut Vec<String>,
        ui: &mut egui::Ui,
    ) -> anyhow::Result<()> {
        if (*self.handle.status_rx.borrow()).is_disconnected() {
            if let Some(credentials) = &self.credentials {
                if ui.button("connect").clicked() {
                    self.handle
                        .connect(credentials.clone())
                        .context("Failed to connect")?;
                }
            } else if ui.button("Request Credentials").clicked() {
                messages.push(
                    serde_json::to_string(&SignalingCommand::from(
                        SignalingModuleCommand::LiveKit(LiveKitCommand::CreateNewAccessToken),
                    ))
                    .context("LiveKit command failed to serializable")?,
                );
            }
        } else if ui.button("disconnect").clicked() {
            self.handle.disconnect().context("Failed to disconnect")?;
        }
        Ok(())
    }

    fn handle_credentials(&mut self, credentials: &Credentials) {
        self.credentials.replace(credentials.clone());
    }
}

impl SignalingPlugin for LiveKitPlugin {
    fn handle_events(
        &mut self,
        _settings: &mut crate::settings::DuiSettings,
        received: &[Received],
    ) -> Vec<String> {
        log::trace!("received {} events", received.len());
        for msg in received {
            let res = match msg {
                Received::SignalingEvent(SignalingEvent {
                    content: SignalingModuleEvent::Core(CoreEvent::JoinSuccess(join)),
                    ..
                }) => self.handle_join_success(join),
                Received::SignalingEvent(SignalingEvent {
                    content: SignalingModuleEvent::LiveKit(LiveKitEvent::Error(e)),
                    ..
                }) => self.handle_livekit_error(e),
                Received::SignalingEvent(SignalingEvent {
                    content: SignalingModuleEvent::LiveKit(LiveKitEvent::Credentials(credentials)),
                    ..
                }) => {
                    self.handle_credentials(credentials);
                    Ok(())
                }
                _ => Ok(()),
            };

            if let Err(e) = res {
                log::warn!("{}", e);
            }
        }

        let mut next_event = self.handle.recv_event();
        while let Ok(Some(event)) = next_event {
            self.handle_runner_event(event);
            next_event = self.handle.recv_event();
        }
        if let Err(e) = next_event {
            log::error!("LiveKitRunner gone! {}", e);
            return Vec::new();
        }

        Vec::new()
    }

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _settings: &mut crate::settings::DuiSettings,
    ) -> Vec<String> {
        let mut messages = Vec::new();
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                self.connection_status_ui(ui);
                if let Err(e) = self.toggle_connection_ui(&mut messages, ui) {
                    log::error!("{e:?}");
                }
            });

            ui.separator();
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .max_width(ui.available_width())
                .show(ui, |ui| {
                    egui::Grid::new("Message Grid")
                        .striped(true)
                        .num_columns(1)
                        .show(ui, |ui| {
                            for msg in &self.events {
                                egui::Label::new(msg).wrap().ui(ui);
                                ui.end_row();
                            }
                        });
                });
        });
        messages
    }

    fn name(&self) -> &str {
        "LiveKit"
    }

    fn shortcut(&self) -> Option<&egui::KeyboardShortcut> {
        Some(&TOGGLE_LIVEKIT_WINDOW_SHORTCUT)
    }
}
