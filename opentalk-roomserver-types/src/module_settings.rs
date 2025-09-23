// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{collections::BTreeMap, fmt::Debug};

use opentalk_types_common::{
    modules::{ModuleId, module_id},
    utils::ExampleData,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::json;

pub trait SignalingModuleSettings: Serialize + DeserializeOwned + Debug {
    const NAMESPACE: ModuleId;
}

/// A struct containing settings for multiple signaling modules, each associated with the module's
/// namespace.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(ModuleSettings::example_data())))]
pub struct ModuleSettings(BTreeMap<ModuleId, serde_json::Value>);

impl ModuleSettings {
    /// Create a new empty [`ModuleSettings`].
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Get the settings for a specific module
    pub fn get<T: SignalingModuleSettings>(&self) -> Result<Option<T>, serde_json::Error> {
        self.0
            .get(&T::NAMESPACE)
            .map(|m| serde_json::from_value(m.clone()))
            .transpose()
    }

    /// Set the settings for a specific module
    ///
    /// If an entry with the namespace already exists, it will be overwritten.
    pub fn insert<T: SignalingModuleSettings>(
        &mut self,
        data: &T,
    ) -> Result<(), serde_json::Error> {
        self.0.insert(T::NAMESPACE, serde_json::to_value(data)?);
        Ok(())
    }
}

impl ExampleData for ModuleSettings {
    fn example_data() -> Self {
        Self(BTreeMap::from([(
            module_id!("livekit"),
            json!({
                "public_url": "http://localhost:7880",
                "service_url": "http://localhost:7880",
                "api_key": "devkey",
                "api_secret": "secret",
            }),
        )]))
    }
}
