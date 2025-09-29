// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use chrono::{DateTime, Utc};
use opentalk_types_common::{
    module_resources::ModuleResourceId, modules::ModuleId, rooms::RoomId, tenants::TenantId,
    users::UserId,
};

/// A filter that can be applied when selecting module resources
#[derive(Default, Clone)]
pub struct ModuleResourceFilter {
    pub created_by: Option<UserId>,
    pub tag: Option<String>,
}

/// A stored resource created by a module
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ModuleResource {
    pub id: ModuleResourceId,
    pub tenant_id: TenantId,
    pub room_id: RoomId,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub namespace: ModuleId,
    pub tag: Option<String>,
    pub data: serde_json::Value,
}

/// A json patch operations based on [RFC6902](https://www.rfc-editor.org/rfc/rfc6902#section-4)
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ModuleResourceOperation {
    /// Json patch *add* operation.
    Add {
        /// The path where the value is added.
        path: String,

        /// The value that is added.
        value: serde_json::Value,
    },

    /// Json patch *remove* operation.
    Remove {
        /// The path from where the value is removed.
        path: String,
    },

    /// Json patch *replace* operation.
    Replace {
        /// The path where the value is replaced.
        path: String,

        /// The value by which the existing value is replaced.
        value: serde_json::Value,
    },

    /// Json patch *move* operation.
    Move {
        /// The location from which the value is moved.
        from: String,

        /// The location to which the value is moved.
        path: String,
    },

    /// Json patch *copy* operation.
    Copy {
        /// The location from which the value is copied.
        from: String,

        /// The location to which the value is copied.
        path: String,
    },

    /// Json patch *test* operation.
    Test {
        /// The path of the value that is compared.
        path: String,

        /// The value with which to compare the specified location.
        value: serde_json::Value,
    },
}
