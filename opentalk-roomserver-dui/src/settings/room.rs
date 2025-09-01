// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_types::{
    client_parameters::{ClientKind, ClientParameters, Role},
    public_user_profile::PublicUserProfile,
    room_parameters::RoomParameters,
};
use opentalk_types_common::{
    modules::ModuleId,
    tariffs::{TariffId, TariffModuleResource, TariffResource},
    time::TimeZone,
    users::{UserId, UserInfo},
    utils::ExampleData,
};
use opentalk_types_signaling::ModuleData;

pub fn alice_profile() -> PublicUserProfile {
    PublicUserProfile {
        id: UserId::from_u128(0xa11ce),
        email: "alice@example.com".to_string(),
        user_info: UserInfo {
            title: "M.Sc.".parse().expect("Valid title"),
            firstname: "Alice".to_string(),
            lastname: "Aal".to_string(),
            display_name: "Alice the angry".parse().expect("Valid DisplayName"),
            avatar_url: "https://example.com/avatar-of-alice".to_string(),
        },
        timezone: TimeZone::example_data(),
    }
}

pub fn bob_profile() -> PublicUserProfile {
    PublicUserProfile {
        id: UserId::from_u128(0xb0b),
        email: "bob@example.com".to_string(),
        user_info: UserInfo {
            title: "".parse().expect("Valid title"),
            firstname: "Bob".to_string(),
            lastname: "Barsch".to_string(),
            display_name: "Bob the bold".parse().expect("Valid DisplayName"),
            avatar_url: "https://example.com/avatar-of-bob".to_string(),
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
            modules: [("echo", TariffModuleResource::default())]
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

pub fn alice_client_parameters(connection: usize) -> ClientParameters {
    ClientParameters {
        device_secret: format!("v3rys3cr3tD3v1ce5tr1ng-alice-{connection}")
            .parse()
            .expect("secret must be valid"),
        kind: ClientKind::Registered {
            profile: alice_profile(),
        },
        role: Role::Moderator,
    }
}

pub fn bob_client_parameters(connection: usize) -> ClientParameters {
    ClientParameters {
        device_secret: format!("v3rys3cr3tD3v1ce5tr1ng-bob-{connection}")
            .parse()
            .expect("secret must be valid"),
        kind: ClientKind::Registered {
            profile: bob_profile(),
        },
        role: Role::User,
    }
}

pub fn gustav_client_parameters(connection: usize) -> ClientParameters {
    ClientParameters {
        device_secret: format!("v3rys3cr3tD3v1ce5tr1ng-gustav-{connection}")
            .parse()
            .expect("secret must be valid"),
        kind: ClientKind::Guest {
            display_name: "Gustav"
                .parse()
                .expect("Gustav must be a valid DisplayName"),
        },
        role: Role::User,
    }
}
