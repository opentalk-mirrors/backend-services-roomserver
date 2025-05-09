// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use egui::{Button, Label, RichText, TextEdit};
use egui_extras::{Column, StripBuilder, TableBuilder};
use opentalk_roomserver_types::{
    client_parameters::ClientParameters, room_parameters::RoomParameters,
};
use opentalk_types_common::rooms::RoomId;

use super::{
    TransitionToView,
    shortcuts::SUBMIT_SHORTCUT,
    style::{InvalidInputStyle as _, delete_btn, delete_mode_btn},
};
use crate::{app::connection_config::parameter_widget::ParameterWidget, settings::DuiSettings};

mod parameter_widget;

#[derive(Debug)]
pub struct ConnectionConfigView {
    selected_room_id_index: usize,
    new_room_id: String,
    new_room_id_name: String,

    room_parameters_select: ParameterWidget<RoomParameters>,
    client_parameters_select: ParameterWidget<ClientParameters>,

    // wether or not to show delete buttons
    delete_mode: bool,
}

impl ConnectionConfigView {
    pub fn new(settings: &DuiSettings) -> Self {
        Self {
            selected_room_id_index: 0,
            new_room_id: RoomId::generate().to_string(),
            new_room_id_name: String::new(),

            room_parameters_select: ParameterWidget::new(
                "Room Parameters".to_string(),
                serde_json::to_string_pretty(&settings.default_room_parameters())
                    .expect("RoomParameters are serializable"),
            ),
            client_parameters_select: ParameterWidget::new(
                "Client Parameters".to_string(),
                serde_json::to_string_pretty(&settings.default_client_parameters())
                    .expect("ClientParameters are serializable"),
            ),

            delete_mode: false,
        }
    }

    pub fn menu_ui(&mut self, ui: &mut egui::Ui) {
        delete_mode_btn(ui, &mut self.delete_mode);
    }

    pub(crate) fn ui(
        &mut self,
        settings: &mut DuiSettings,
        ui: &mut egui::Ui,
    ) -> Option<TransitionToView> {
        let mut transition_request = None;
        StripBuilder::new(ui)
            .size(egui_extras::Size::relative(0.2))
            .size(egui_extras::Size::relative(0.35))
            .size(egui_extras::Size::relative(0.35))
            .size(egui_extras::Size::remainder())
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    self.room_id_ui(settings, ui);
                });

                strip.strip(|builder| {
                    self.room_parameters_select.ui(
                        &mut settings.room_parameters,
                        self.delete_mode,
                        builder,
                    );
                });

                strip.strip(|builder| {
                    self.client_parameters_select.ui(
                        &mut settings.client_parameters,
                        self.delete_mode,
                        builder,
                    );
                });

                strip.cell(|ui| {
                    let connect_btn = Button::new("Connect")
                        .shortcut_text(ui.ctx().format_shortcut(&SUBMIT_SHORTCUT));
                    if let (Some(room_id), Ok(room_parameters), Ok(client_parameters)) = (
                        settings
                            .room_ids
                            .get(self.selected_room_id_index)
                            .map(|(_, r)| *r),
                        &self.room_parameters_select.parsed,
                        &self.client_parameters_select.parsed,
                    ) {
                        if ui.add(connect_btn).clicked()
                            || ui.ctx().input_mut(|i| i.consume_shortcut(&SUBMIT_SHORTCUT))
                        {
                            transition_request = Some(TransitionToView::Connecting {
                                room_id,
                                client_parameters: client_parameters.clone().into(),
                                room_parameters: room_parameters.clone().into(),
                            });
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
                        if delete_btn(ui, self.delete_mode).clicked() {
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

            // when we delete something, the selected index might "shift" and the selection changed.
            // Decrease by 1 to avoid that unwanted change.
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
}
