// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use egui::{Button, Color32, RichText, TextEdit};
use opentalk_roomserver_types::{
    client_parameters::{ClientKind, ClientParameters, Role},
    room_parameters::RoomParameters,
};
use opentalk_types_api_v1::users::PublicUserProfile;
use opentalk_types_common::{
    modules::ModuleId,
    rooms::RoomId,
    tariffs::{TariffId, TariffModuleResource, TariffResource},
    users::{UserId, UserInfo},
};

use super::{
    json_edit::json_editor,
    shortcuts::SUBMIT_SHORTCUT,
    style::{InvalidInputStyle, SECTION_SPACE_HIGHT},
    TransitionToView,
};

fn alice_profile() -> PublicUserProfile {
    PublicUserProfile {
        id: UserId::from_u128(0xf53bc453_64f3_471f_bc4b_a1adcc8a392d),
        email: "alice@example.com".to_string(),
        user_info: UserInfo {
            title: "".parse().expect("valid user title"),
            firstname: "Alice".to_string(),
            lastname: "Adams".to_string(),
            display_name: "Alice Adams".parse().expect("valid display name"),
            avatar_url: "https://gravatar.com/avatar/c160f8cc69a4f0bf2b0362752353d060".to_string(),
        },
    }
}

fn default_room_parameters() -> String {
    serde_json::to_string_pretty(&RoomParameters {
        created_by: alice_profile(),
        password: None,
        waiting_room: false,
        call_in: None,
        event: None,
        invite_code: None,
        tariff: TariffResource {
            id: TariffId::from_u128(0x2da2b825_6db9_4dc4_b9e6_b4fd64e66a16),
            name: "Starter tariff".to_string(),
            quotas: Default::default(),
            modules: [("ping", TariffModuleResource::default())]
                .into_iter()
                .map(|(module, resource)| {
                    (
                        module.parse::<ModuleId>().expect("valid module id"),
                        resource,
                    )
                })
                .collect(),
        },
        streaming_links: vec![],
        e2e_encryption: false,
    })
    .expect("RoomParameters are serializable")
}

fn default_client_parameters() -> String {
    serde_json::to_string_pretty(&ClientParameters {
        device_secret: "this is not secure".to_string(),
        kind: ClientKind::Registered {
            profile: alice_profile(),
        },
        role: Role::Moderator,
    })
    .expect("ClientParameter are serializable")
}

#[derive(Debug)]
pub struct ConnectionConfigView {
    room_id: String,
    room_parameters: String,
    client_parameters: String,
}

impl ConnectionConfigView {
    pub fn new() -> Self {
        Self {
            room_id: uuid::Uuid::new_v4().to_string(),
            room_parameters: default_room_parameters(),
            client_parameters: default_client_parameters(),
        }
    }

    pub(crate) fn ui(&mut self, ui: &mut egui::Ui) -> Option<TransitionToView> {
        let room_id = self.room_id.parse::<RoomId>();
        let room_parameters = serde_json::from_str::<RoomParameters>(self.room_parameters.as_str());
        let client_parameters =
            serde_json::from_str::<ClientParameters>(self.client_parameters.as_str());

        ui.horizontal(|ui| {
            let name_label = ui.label("Room ID: ");
            let mut edit = TextEdit::singleline(&mut self.room_id);
            if room_id.is_err() {
                edit = edit.invalid_input_style();
            }
            ui.add(edit).labelled_by(name_label.id);
        });

        ui.push_id("room-parameters", |ui| {
            ui.horizontal(|ui| {
                ui.heading("Room Parameters:");
                if ui.button("Reset").clicked() {
                    self.room_parameters = default_room_parameters();
                }
                if let Some(e) = room_parameters.as_ref().err() {
                    ui.label(e.to_string());
                }
            });
            json_editor(ui, &mut self.room_parameters);
        });

        ui.add_space(SECTION_SPACE_HIGHT);

        ui.push_id("client-parameters", |ui| {
            ui.horizontal(|ui| {
                ui.heading("Client Parameters:");
                if ui.button("Reset").clicked() {
                    self.client_parameters = default_client_parameters();
                }
                if let Some(e) = client_parameters.as_ref().err() {
                    ui.label(RichText::new(e.to_string()).color(Color32::RED));
                }
            });
            json_editor(ui, &mut self.client_parameters);
        });

        let connect_btn =
            Button::new("Connect").shortcut_text(ui.ctx().format_shortcut(&SUBMIT_SHORTCUT));
        if let (Ok(room_id), Ok(room_parameters), Ok(client_parameters)) =
            (room_id, room_parameters, client_parameters)
        {
            if ui.add(connect_btn).clicked()
                || ui.ctx().input_mut(|i| i.consume_shortcut(&SUBMIT_SHORTCUT))
            {
                Some(TransitionToView::Connecting {
                    room_id,
                    client_parameters: client_parameters.into(),
                    room_parameters: room_parameters.into(),
                })
            } else {
                None
            }
        } else {
            ui.add_enabled(false, connect_btn);
            None
        }
    }
}

impl Default for ConnectionConfigView {
    fn default() -> Self {
        Self::new()
    }
}
