// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::fmt::Debug;

use opentalk_types_api_v1::users::PublicUserProfile;
use opentalk_types_common::{
    call_in::CallInInfo, shared_folders::SharedFolder, streaming::StreamingLink,
    tariffs::TariffResource,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct RoomParameters {
    pub created_by: PublicUserProfile,

    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub password: Option<String>,

    pub waiting_room: bool,

    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub event: Option<EventInfo>,

    pub tariff: TariffResource,
}

impl Debug for RoomParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RoomParameters")
            .field("created_by", &self.created_by)
            .field("password", &"<REDACTED>")
            .field("waiting_room", &self.waiting_room)
            .field("event", &self.event)
            .field("tariff", &self.tariff)
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct EventInfo {
    pub title: String,

    pub description: String,

    pub is_adhoc: bool,

    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub invite_code_id: Option<String>,

    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub call_in: Option<CallInInfo>,

    pub streaming_links: Vec<StreamingLink>,

    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub shared_folder: Option<SharedFolder>,
}

#[cfg(test)]
mod tests {
    use opentalk_types_common::utils::ExampleData;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn room_parameters() {
        let params = RoomParameters {
            created_by: PublicUserProfile::example_data(),
            password: Default::default(),
            waiting_room: Default::default(),
            event: Default::default(),
            tariff: TariffResource::example_data(),
        };
        let json = json!({
            "created_by": {
                "avatar_url": "https://gravatar.com/avatar/c160f8cc69a4f0bf2b0362752353d060",
                "display_name": "Alice Adams",
                "email": "alice@example.com",
                "firstname": "Alice",
                "id": "00000000-0000-0000-0000-0000000a11c3",
                "lastname": "Adams",
                "title": "",
            },
            "password": None::<String>,
            "waiting_room": false,
            "event": None::<String>,
            "tariff": {
                "id": "00000000-0000-0000-0000-000000000000",
                "name": "Starter tariff",
                "quotas": {
                    "max_storage": 50000
                },
                "modules": {
                    "chat": {
                        "features": []
                    },
                    "core": {
                        "features": []
                    },
                    "media": {
                        "features": []
                    },
                    "moderation": {
                        "features": []
                    },
                    "recording": {
                        "features": [ "record" ]
                    }
                }
            }
        });

        // serialization
        assert_eq!(json.clone(), serde_json::to_value(params.clone()).unwrap());

        // deserialization
        assert_eq!(params, serde_json::from_value(json).unwrap(),);
    }

    #[test]
    fn event_info() {
        let info = EventInfo {
            title: Default::default(),
            description: Default::default(),
            is_adhoc: Default::default(),
            invite_code_id: Default::default(),
            call_in: Some(CallInInfo::example_data()),
            streaming_links: vec![StreamingLink::example_data()],
            shared_folder: Some(SharedFolder::example_data()),
        };
        let json = json!({
            "title": "",
            "description": "",
            "is_adhoc": false,
            "invite_code_id": None::<String>,
            "call_in": {
                "tel": "+555-123-456-789",
                "id": "1234567890",
                "password": "0987654321"
            },
            "streaming_links": [
                {
                    "name": "My OwnCast Stream",
                    "url": "https://owncast.example.com/mystream",
                }
            ],
            "shared_folder": {
                "read": {
                    "password": "v3rys3cr3t",
                    "url": "https://cloud.example.com/shares/abc123",
                }
            },
        });

        // serialization
        assert_eq!(json.clone(), serde_json::to_value(info.clone()).unwrap());

        // deserialization
        assert_eq!(info, serde_json::from_value(json).unwrap());
    }
}
