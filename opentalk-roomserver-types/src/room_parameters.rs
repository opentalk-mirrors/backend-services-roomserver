// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{fmt::Debug, str::FromStr};

use chrono::TimeDelta;
use opentalk_types_api_v1::users::PublicUserProfile;
use opentalk_types_common::{
    call_in::CallInInfo,
    events::{EventDescription, EventId, EventTitle},
    rooms::{invite_codes::InviteCode, RoomPassword},
    shared_folders::SharedFolder,
    streaming::StreamingLink,
    tariffs::{QuotaType, TariffResource},
    time::Timestamp,
    utils::ExampleData,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(RoomParameters::example_data())))]
pub struct RoomParameters {
    pub created_by: PublicUserProfile,

    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub password: Option<RoomPassword>,

    pub waiting_room: bool,

    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub call_in: Option<CallInInfo>,

    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub event: Option<EventContext>,

    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub invite_code: Option<InviteCode>,

    pub tariff: TariffResource,

    pub streaming_links: Vec<StreamingLink>,

    /// Indicates whether the meeting room should have e2e encryption enabled.
    pub e2e_encryption: bool,
}

impl RoomParameters {
    pub fn calc_time_limit_quota(&self, start_time: Timestamp) -> Option<Timestamp> {
        let remaining_secs = self
            .tariff
            .quotas
            .get(&QuotaType::RoomTimeLimitSecs)
            .map(|secs| i64::try_from(*secs).unwrap_or(i64::MAX))?;

        start_time
            .checked_add_signed(TimeDelta::seconds(remaining_secs))
            .map(Timestamp::from)
    }
}

impl Debug for RoomParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RoomParameters")
            .field("created_by", &self.created_by)
            .field("password", &"<REDACTED>")
            .field("waiting_room", &self.waiting_room)
            .field("call_in", &self.call_in)
            .field("event", &self.event)
            .field("invite_code", &self.invite_code)
            .field("tariff", &self.tariff)
            .field("streaming_links", &self.streaming_links)
            .finish()
    }
}

impl ExampleData for RoomParameters {
    fn example_data() -> Self {
        Self {
            created_by: PublicUserProfile::example_data(),
            password: Some(RoomPassword::from_str("1234").unwrap()),
            waiting_room: false,
            call_in: Some(CallInInfo::example_data()),
            event: Some(EventContext::example_data()),
            invite_code: Some(InviteCode::example_data()),
            tariff: TariffResource::example_data(),
            streaming_links: vec![StreamingLink::example_data()],
            e2e_encryption: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(EventContext::example_data())))]
pub struct EventContext {
    pub id: EventId,

    pub title: EventTitle,

    pub description: EventDescription,

    pub is_adhoc: bool,

    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub shared_folder: Option<SharedFolder>,
}

impl ExampleData for EventContext {
    fn example_data() -> Self {
        Self {
            id: EventId::example_data(),
            title: EventTitle::example_data(),
            description: EventDescription::example_data(),
            is_adhoc: false,
            shared_folder: Some(SharedFolder::example_data()),
        }
    }
}

#[cfg(test)]
mod tests {
    use opentalk_types_common::utils::ExampleData;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn room_parameters() {
        let params = RoomParameters::example_data();
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
            "password": "",
            "call_in": {
                "tel": "+555-123-456-789",
                "id": "1234567890",
                "password": "0987654321"
            },
            "waiting_room": false,
            "event": {
                "description": "The Weekly Team Event",
                "id": "00000000-0000-0000-0000-004433221100",
                "is_adhoc": false,
                "shared_folder": {
                    "read": {
                        "password": "v3rys3cr3t",
                        "url": "https://cloud.example.com/shares/abc123",
                    },
                },
                "title": "Team Event",
            },
            "invite_code": "00000000-0000-0000-0000-0000deadbeef",
            "password": "1234",
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
            },
            "streaming_links": [{"name": "My OwnCast Stream", "url": "https://owncast.example.com/mystream"}],
            "e2e_encryption": false
        });

        // serialization
        assert_eq!(json.clone(), serde_json::to_value(params.clone()).unwrap());

        // deserialization
        assert_eq!(params, serde_json::from_value(json).unwrap(),);
    }

    #[test]
    fn event_info() {
        let info = EventContext::example_data();
        let json = json!({
            "id": "00000000-0000-0000-0000-004433221100",
            "title": "Team Event",
            "description": "The Weekly Team Event",
            "is_adhoc": false,
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
