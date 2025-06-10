// SPDX-License-Identifier: EUPL-1.2
//
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::sync::Arc;

use serde::{Deserialize, Serialize, Serializer};
use serde_json::value::RawValue;

/// Type to deal with opaque JSON values.
///
/// Some scenarios require sending the same value to a large amount of participants,
/// which is why the value is reference counted and therefore cheap to clone.
#[derive(Debug, Clone)]
pub struct SharedRawJson {
    inner: Arc<RawValue>,
}

impl From<Box<RawValue>> for SharedRawJson {
    fn from(value: Box<RawValue>) -> Self {
        Self {
            inner: value.into(),
        }
    }
}

impl Serialize for SharedRawJson {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        RawValue::serialize(&*self.inner, serializer)
    }
}

impl<'de> Deserialize<'de> for SharedRawJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <Box<RawValue>>::deserialize(deserializer).map(Self::from)
    }
}
