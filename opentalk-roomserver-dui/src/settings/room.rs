// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_types::{
    client_parameters::{ClientKind, ClientParameters, Role},
    public_user_profile::PublicUserProfile,
    room_parameters::RoomParameters,
};
use opentalk_types_common::{
    modules::ModuleId,
    roomserver::DeviceSecret,
    tariffs::{TariffId, TariffModuleResource, TariffResource},
    time::TimeZone,
    users::{UserId, UserInfo},
    utils::ExampleData,
};
use opentalk_types_signaling::ModuleData;

pub fn alice_profile() -> PublicUserProfile {
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
        timezone: TimeZone::example_data(),
    }
}

pub fn default_room_parameters() -> RoomParameters {
    RoomParameters {
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
        module_data: ModuleData::example_data(),
    }
}

pub fn default_client_parameters() -> ClientParameters {
    ClientParameters {
        device_secret: DeviceSecret::example_data(),
        kind: ClientKind::Registered {
            profile: alice_profile(),
        },
        role: Role::Moderator,
    }
}
