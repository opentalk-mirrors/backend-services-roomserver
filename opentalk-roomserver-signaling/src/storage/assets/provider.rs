// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use opentalk_types_common::assets::AssetId;

use crate::storage::assets::{AssetMetaData, StorageContext, StorageError, UploadResult};

#[async_trait]
pub trait AssetStorageProvider: Send + Sync + Debug {
    /// Uploads an asset to the storage backend
    async fn upload_asset(
        &self,
        asset: Vec<u8>,
        metadata: AssetMetaData,
        context: &StorageContext,
    ) -> UploadResult;

    /// Uploads a chunk of data to the storage backend
    async fn upload_chunk(
        &self,
        id: AssetId,
        chunk: &[u8],
        context: &StorageContext,
    ) -> Result<(), StorageError>;

    /// Finalizes an upload of one or multiple chunks
    async fn finalize_upload(
        &self,
        id: AssetId,
        metadata: AssetMetaData,
        context: &StorageContext,
    ) -> UploadResult;

    async fn remaining_quota(&self, context: &StorageContext) -> Option<u64>;

    fn into_any(self: Arc<Self>) -> Arc<dyn std::any::Any + Send + Sync>;
}
