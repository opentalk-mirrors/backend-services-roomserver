// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types::api::v1::users::PublicUserProfile;
use opentalk_types_common::{
    call_in::CallInInfo, shared_folders::SharedFolder, streaming::StreamingLink,
    tariffs::TariffResource,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoomParameters {
    pub created_by: PublicUserProfile,
    pub password: Option<String>,
    pub waiting_room: bool,
    pub event: Option<EventInfo>,
    pub tariff: TariffResource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventInfo {
    pub title: String,
    pub description: String,
    pub is_adhoc: bool,
    pub invite_code_id: Option<String>,
    pub call_in: Option<CallInInfo>,
    pub streaming_links: Vec<StreamingLink>,
    pub shared_folder: Option<SharedFolder>,
}
