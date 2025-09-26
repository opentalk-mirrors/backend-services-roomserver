// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use async_trait::async_trait;
use opentalk_types_common::{
    module_resources::ModuleResourceId, modules::ModuleId, rooms::RoomId, users::UserId,
};

use crate::storage::module_resources::{
    Error, ModuleResource, ModuleResourceFilter, ModuleResourceOperation,
};

/// The internal interface to create or retrieve [`ModuleResource`]s
///
/// Can be implemented by some backend that allows to store module resources
#[async_trait]
pub trait ModuleResourceProvider: Send + Sync {
    /// Create a new [`ModuleResource`]
    ///
    /// Returns the created resource
    async fn create(&self, resource: NewModuleResource) -> Result<ModuleResource, Error>;

    /// Get all [`ModuleResource`]s where the filter applies
    async fn get(&self, filter: InternalModuleResourceFilter)
    -> Result<Vec<ModuleResource>, Error>;

    /// Apply the given json operations to all [`ModuleResource`]s where the filter applies
    ///
    /// When one patch operation fails on any resource, none of the resources will be modified
    async fn patch(
        &self,
        filter: InternalModuleResourceFilter,
        operations: Vec<ModuleResourceOperation>,
    ) -> Result<Vec<ModuleResource>, Error>;

    /// Delete all [`ModuleResource`]s where the filter applies
    async fn delete(
        &self,
        filter: InternalModuleResourceFilter,
    ) -> Result<Vec<ModuleResource>, Error>;
}

/// Necessary values to create a new module resource with the [`ModuleResourceProvider`]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewModuleResource {
    /// The id of the room to which the module resource belongs to
    pub room_id: RoomId,

    /// The id of the user who created the module resource.
    pub created_by: UserId,

    /// The namespace of the module resource.
    pub namespace: ModuleId,

    /// An optional tag for the module resource, may be used by the corresponding module.
    pub tag: Option<String>,

    /// The module resource data.
    pub data: serde_json::Value,
}

/// A filter to select module resources by specific values
#[derive(Default, Clone)]
pub struct InternalModuleResourceFilter {
    pub id: Option<ModuleResourceId>,
    pub room_id: Option<RoomId>,
    pub created_by: Option<UserId>,
    pub namespace: Option<ModuleId>,
    pub tag: Option<String>,
}

impl InternalModuleResourceFilter {
    /// Create a new resource filter for the given room and namespace
    pub fn new(room_id: RoomId, namespace: ModuleId) -> Self {
        Self {
            id: None,
            room_id: Some(room_id),
            created_by: None,
            namespace: Some(namespace),
            tag: None,
        }
    }

    /// Returns true when all values of the filter match the given resource values
    pub fn applies_to(&self, resource: &ModuleResource) -> bool {
        if let Some(id) = &self.id
            && &resource.id != id
        {
            return false;
        }

        if let Some(room_id) = &self.room_id
            && &resource.room_id != room_id
        {
            return false;
        }

        if let Some(created_by) = &self.created_by
            && &resource.created_by != created_by
        {
            return false;
        }

        if let Some(namespace) = &self.namespace
            && &resource.namespace != namespace
        {
            return false;
        }

        if let Some(tag) = &self.tag
            && resource.tag.as_ref() != Some(tag)
        {
            return false;
        }

        true
    }

    pub fn id(mut self, id: ModuleResourceId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn room_id(mut self, room_id: RoomId) -> Self {
        self.room_id = Some(room_id);
        self
    }

    pub fn created_by(mut self, user_id: UserId) -> Self {
        self.created_by = Some(user_id);
        self
    }

    pub fn namespace(mut self, namespace: ModuleId) -> Self {
        self.namespace = Some(namespace);
        self
    }

    pub fn tag(mut self, tag: Option<String>) -> Self {
        self.tag = tag;
        self
    }
}

impl From<ModuleResourceFilter> for InternalModuleResourceFilter {
    fn from(other: ModuleResourceFilter) -> Self {
        InternalModuleResourceFilter {
            id: None,
            room_id: None,
            created_by: other.created_by,
            namespace: None,
            tag: other.tag,
        }
    }
}
