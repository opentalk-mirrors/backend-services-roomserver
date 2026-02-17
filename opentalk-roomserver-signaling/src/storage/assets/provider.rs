// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{any::Any, fmt::Debug};

use async_trait::async_trait;
use bytes::Bytes;
use futures::stream::BoxStream;

use crate::storage::assets::{AssetMetaData, StorageContext, UploadResult};

pub type AssetStream = BoxStream<'static, Result<Bytes, AssetLoadError>>;

#[derive(Debug)]
pub struct AssetLoadError {
    pub source: Box<dyn std::error::Error + Send + Sync>,
}

// this is is required to transform the AssetStream into a reqwest Body.
impl From<AssetLoadError> for Box<dyn std::error::Error + Send + Sync> {
    fn from(err: AssetLoadError) -> Self {
        err.source
    }
}

#[async_trait]
pub trait AssetStorageProvider: Send + Sync + Debug + Any {
    /// Uploads an asset to the storage backend
    async fn upload_asset(
        &self,
        asset: AssetStream,
        metadata: AssetMetaData,
        context: &StorageContext,
    ) -> UploadResult;

    async fn remaining_quota(&self, context: &StorageContext) -> Option<u64>;
}
