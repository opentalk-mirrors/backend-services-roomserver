// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use egui::{Button, Label, RichText, TextEdit};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use opentalk_roomserver_types::{
    client_parameters::ClientParameters, room_parameters::RoomParameters,
};
use opentalk_types_common::rooms::RoomId;

use super::{
    TransitionToView, json_edit::json_editor, shortcuts::SUBMIT_SHORTCUT,
    style::InvalidInputStyle as _,
};
use crate::settings::DuiSettings;

#[derive(Debug)]
pub struct ConnectionConfigView {
    selected_room_id_index: usize,
    new_room_id: String,
    new_room_id_name: String,

    new_room_parameters_name: String,
    selected_room_parameters_index: usize,
    room_parameters: String,
    client_parameters: String,
}

impl ConnectionConfigView {
    pub fn new(settings: &DuiSettings) -> Self {
        Self {
            selected_room_id_index: 0,
            new_room_id: RoomId::generate().to_string(),
            new_room_id_name: String::new(),

            new_room_parameters_name: String::new(),
            selected_room_parameters_index: 0,
            room_parameters: serde_json::to_string_pretty(&settings.default_room_parameters())
                .expect("RoomParameters are serializable"),
            client_parameters: serde_json::to_string_pretty(&settings.default_client_parameters())
                .expect("ClientParameters are serializable"),
        }
    }

    pub(crate) fn ui(
        &mut self,
        settings: &mut DuiSettings,
        ui: &mut egui::Ui,
    ) -> Option<TransitionToView> {
        let room_parameters = serde_json::from_str::<RoomParameters>(self.room_parameters.as_str());
        let client_parameters =
            serde_json::from_str::<ClientParameters>(self.client_parameters.as_str());

        let mut transition_request = None;
        StripBuilder::new(ui)
            .size(egui_extras::Size::relative(0.3))
            .size(egui_extras::Size::relative(0.3))
            .size(egui_extras::Size::relative(0.3))
            .size(egui_extras::Size::remainder())
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    self.room_id_ui(settings, ui);
                });

                strip.strip(|builder| {
                    self.room_parameter_ui(&room_parameters, settings, builder);
                });

                strip.cell(|ui| {
                    self.client_parameter_ui(&client_parameters, ui);
                });

                strip.cell(|ui| {
                    let connect_btn = Button::new("Connect")
                        .shortcut_text(ui.ctx().format_shortcut(&SUBMIT_SHORTCUT));
                    if let (Some(room_id), Ok(room_parameters), Ok(client_parameters)) = (
                        settings
                            .room_ids
                            .get(self.selected_room_id_index)
                            .map(|(_, r)| *r),
                        room_parameters,
                        client_parameters,
                    ) {
                        if ui.add(connect_btn).clicked()
                            || ui.ctx().input_mut(|i| i.consume_shortcut(&SUBMIT_SHORTCUT))
                        {
                            transition_request = Some(TransitionToView::Connecting {
                                room_id,
                                client_parameters: client_parameters.into(),
                                room_parameters: room_parameters.into(),
                            })
                        }
                    } else {
                        ui.add_enabled(false, connect_btn);
                    }
                });
            });

        transition_request
    }

    fn room_id_ui(&mut self, settings: &mut DuiSettings, ui: &mut egui::Ui) {
        let available_height = ui.available_height();

        let text_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);

        let parsed_room_id = self.new_room_id.parse::<RoomId>();

        ui.heading("Room ID");

        let mut delete_room_id = None;

        TableBuilder::new(ui)
            .striped(true)
            .sense(egui::Sense::click())
            .max_scroll_height(available_height)
            .auto_shrink(false)
            .column(Column::auto())
            .column(Column::exact(280.))
            .column(Column::remainder().clip(true))
            .body(|body| {
                body.rows(text_height, settings.room_ids.len(), |mut row| {
                    let row_index = row.index();
                    let Some((name, room_id)) = settings.room_ids.get(row_index) else {
                        return;
                    };

                    row.set_selected(self.selected_room_id_index == row_index);

                    row.col(|ui| {
                        if ui.button("❌").clicked() {
                            delete_room_id = Some(row_index);
                        }
                    });

                    row.col(|ui| {
                        ui.add(Label::new(RichText::new(room_id.to_string()).monospace()));
                    });

                    row.col(|ui| {
                        ui.label(name);
                    });

                    if row.response().clicked() {
                        self.selected_room_id_index = row_index;
                    }
                });
            });

        if let Some(index) = delete_room_id {
            settings.room_ids.remove(index);

            // when we delete something, the selected index might "shift" and the selection changed
            if self.selected_room_id_index > index {
                self.selected_room_id_index = self.selected_room_id_index.saturating_sub(1);
            }
            ui.ctx().request_repaint();
        }

        ui.horizontal(|ui| {
            ui.add(TextEdit::singleline(&mut self.new_room_id_name).hint_text("Room Name"));
            let mut room_id_input =
                TextEdit::singleline(&mut self.new_room_id).hint_text("Room ID");
            if parsed_room_id.is_err() {
                room_id_input = room_id_input.invalid_input_style();
            }
            let res = ui.add(room_id_input);
            if let Err(e) = &parsed_room_id {
                res.on_hover_ui(|ui| {
                    ui.label(e.to_string());
                });
            }
            if ui.button("🎲").clicked() {
                self.new_room_id = RoomId::generate().to_string();
            }
            if ui
                .add_enabled(parsed_room_id.is_ok(), egui::Button::new("save"))
                .clicked()
            {
                if let Ok(room_id) = parsed_room_id {
                    settings
                        .room_ids
                        .push((self.new_room_id_name.clone(), room_id));
                    self.new_room_id_name.clear();
                    self.new_room_id = RoomId::generate().to_string();
                    self.selected_room_id_index = settings.room_ids.len() - 1;
                }
            }
        });
    }

    fn client_parameter_ui(
        &mut self,
        client_parameters: &Result<ClientParameters, serde_json::Error>,
        ui: &mut egui::Ui,
    ) {
        ui.horizontal(|ui| {
            ui.heading("Client Parameters:");
            if let Some(e) = client_parameters.as_ref().err() {
                let err_text = RichText::new(e.to_string()).invalid_input_style();
                ui.label(err_text);
            }
        });
        json_editor(ui, &mut self.client_parameters);
    }

    fn room_parameter_ui(
        &mut self,
        room_parameters: &Result<RoomParameters, serde_json::Error>,
        settings: &mut DuiSettings,
        builder: StripBuilder<'_>,
    ) {
        builder
            .size(egui_extras::Size::initial(20.))
            .size(Size::remainder())
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Room Parameters");
                        ui.add(
                            TextEdit::singleline(&mut self.new_room_parameters_name)
                                .hint_text("Room Name"),
                        );

                        if ui
                            .add_enabled(room_parameters.is_ok(), egui::Button::new("save"))
                            .clicked()
                        {
                            if let Ok(parameters) = room_parameters {
                                settings.room_parameters.push((
                                    self.new_room_parameters_name.clone(),
                                    parameters.clone(),
                                ));
                                self.new_room_parameters_name.clear();
                                self.selected_room_parameters_index =
                                    settings.room_parameters.len() - 1;
                            }
                        }
                        if let Some(e) = room_parameters.as_ref().err() {
                            let err_text = RichText::new(e.to_string()).invalid_input_style();
                            ui.label(err_text);
                        }
                    });
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::relative(0.2))
                        .size(Size::remainder())
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                self.room_parameter_table_ui(settings, ui);
                            });

                            strip.cell(|ui| {
                                json_editor(ui, &mut self.room_parameters);
                            });
                        });
                });
            });
    }

    fn room_parameter_table_ui(&mut self, settings: &mut DuiSettings, ui: &mut egui::Ui) {
        let available_height = ui.available_height();

        let text_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);

        let mut delete_index = None;
        TableBuilder::new(ui)
            .striped(true)
            .sense(egui::Sense::click())
            .max_scroll_height(available_height)
            .auto_shrink(false)
            .column(Column::auto())
            .column(Column::remainder())
            .body(|body| {
                body.rows(text_height, settings.room_parameters.len(), |mut row| {
                    let row_index = row.index();
                    let Some((name, _)) = settings.room_parameters.get(row_index) else {
                        return;
                    };

                    row.set_selected(self.selected_room_parameters_index == row_index);

                    row.col(|ui| {
                        if ui.button("❌").clicked() {
                            delete_index = Some(row_index);
                        }
                    });

                    row.col(|ui| {
                        ui.label(name);
                    });

                    if row.response().clicked() {
                        self.selected_room_parameters_index = row_index;
                        if let Some((_, room_parameters)) = settings.room_parameters.get(row_index)
                        {
                            self.room_parameters = serde_json::to_string_pretty(&room_parameters)
                                .expect("RoomParameters are serializable");
                        }
                    }
                });
            });

        if let Some(index) = delete_index {
            settings.room_parameters.remove(index);
            if self.selected_room_parameters_index > index {
                self.selected_room_parameters_index =
                    self.selected_room_parameters_index.saturating_sub(1);
            }
            ui.ctx().request_repaint();
        }
    }
}
