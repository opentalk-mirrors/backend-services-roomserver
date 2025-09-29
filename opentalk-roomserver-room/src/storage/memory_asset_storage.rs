// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use futures::StreamExt;
use opentalk_roomserver_signaling::storage::{
    StorageContext,
    assets::{
        AssetMetaData, AssetUploaded, StorageError, UploadResult,
        provider::{AssetStorageProvider, AssetStream},
    },
};
use opentalk_types_common::assets::AssetId;
use tokio::sync::Mutex;
use url::Url;

/// A simple storage provider using the local asset system as storage backend.
///
/// This implementation is for testing purposes only. It will be moved to the
/// mocking module and only be used for tests once a real storage provider has
/// been implemented.
#[derive(Debug)]
pub struct MemoryAssetStorage {
    quota: Option<u64>,
    assets: Mutex<HashMap<AssetId, Vec<u8>>>,
}

impl MemoryAssetStorage {
    /// Creates a new [`MemoryAssetStorage`]
    ///
    /// * `quota` - The total size of assets the user is allowed to upload (in bytes)
    pub fn new(quota: Option<u64>) -> Self {
        Self {
            quota,
            assets: Mutex::new(HashMap::new()),
        }
    }

    pub async fn asset(&self, id: AssetId) -> Option<Vec<u8>> {
        self.assets.lock().await.get(&id).cloned()
    }

    pub async fn asset_count(&self) -> usize {
        self.assets.lock().await.len()
    }
}

#[async_trait]
impl AssetStorageProvider for MemoryAssetStorage {
    async fn upload_asset(
        &self,
        mut asset: AssetStream,
        metadata: AssetMetaData,
        context: &StorageContext,
    ) -> UploadResult {
        if self
            .remaining_quota(context)
            .await
            .map(|q| q == 0)
            .unwrap_or(false)
        {
            return Err(StorageError::QuotaReached);
        }

        let mut data = Vec::new();
        while let Some(chunk) = asset.next().await {
            let chunk = chunk.map_err(StorageError::ReadAsset)?;
            data.extend_from_slice(&chunk);
        }

        let id = AssetId::generate();
        self.assets.lock().await.insert(id, data);

        Ok(AssetUploaded {
            id,
            filename: metadata.to_string(),
            remaining_quota: self.remaining_quota(context).await,
            url: Self::asset_url(id, &metadata),
        })
    }

    async fn remaining_quota(&self, _context: &StorageContext) -> Option<u64> {
        if let Some(q) = self.quota {
            let used: usize = self.assets.lock().await.values().map(Vec::len).sum();
            Some(q.saturating_sub(used as u64))
        } else {
            None
        }
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn std::any::Any + Send + Sync> {
        self
    }
}

impl MemoryAssetStorage {
    fn asset_url(id: AssetId, metadata: &AssetMetaData) -> Url {
        let asset_name = format!("{id}_{metadata}");
        let url = format!("file://{asset_name}");
        Url::parse(&url).expect("Parsing url failed")
    }
}

#[cfg(test)]
mod test {
    use anyhow::anyhow;
    use futures::{StreamExt, stream};
    use opentalk_roomserver_signaling::storage::{
        StorageContext,
        assets::{
            AssetMetaData, StorageError,
            provider::{AssetLoadError, AssetStorageProvider as _},
        },
    };
    use opentalk_roomserver_types::breakout::BREAKOUT_MODULE_ID;
    use opentalk_types_common::{
        assets::{FileExtension, asset_file_kind},
        rooms::RoomId,
        time::Timestamp,
    };

    use super::MemoryAssetStorage;

    #[tokio::test]
    async fn upload_asset() {
        let quota = 5 * 1024u64.pow(3);
        let storage = MemoryAssetStorage::new(Some(quota));
        let storage_context = StorageContext {
            room_id: RoomId::from_u128(0x12),
            namespace: BREAKOUT_MODULE_ID,
        };

        let content = b"some file content";

        let asset = stream::iter(vec![Ok(bytes::Bytes::from_static(content))]);
        let name = AssetMetaData {
            kind: asset_file_kind!("text"),
            timestamp: Timestamp::now(),
            // using pdf as extension here because this is the only extension
            // we currently have and it is not worth adding one only for tests
            extension: FileExtension::pdf(),
        };
        let uploaded = storage
            .upload_asset(asset.boxed(), name, &storage_context)
            .await
            .unwrap();
        let produced = storage.asset(uploaded.id).await.unwrap();

        assert_eq!(content.to_vec(), produced);
    }

    #[tokio::test]
    async fn exceed_quota() {
        let quota = 1;
        let storage = MemoryAssetStorage::new(Some(quota));
        let storage_context = StorageContext {
            room_id: RoomId::from_u128(0x12),
            namespace: BREAKOUT_MODULE_ID,
        };

        let content = b"asset that exceeds the quota";

        let asset = stream::iter(vec![Ok(bytes::Bytes::from_static(content))]);
        let name = AssetMetaData {
            kind: asset_file_kind!("text"),
            timestamp: Timestamp::now(),
            extension: FileExtension::pdf(),
        };
        storage
            .upload_asset(asset.boxed(), name, &storage_context)
            .await
            .unwrap();

        let asset = stream::iter(vec![Ok(bytes::Bytes::from_static(content))]);
        let name = AssetMetaData {
            kind: asset_file_kind!("text"),
            timestamp: Timestamp::now(),
            extension: FileExtension::pdf(),
        };
        let produced = storage
            .upload_asset(asset.boxed(), name, &storage_context)
            .await;

        assert!(matches!(produced, Err(StorageError::QuotaReached)));
    }

    #[tokio::test]
    async fn stream_error() {
        let quota = 5 * 1024u64.pow(3);
        let storage = MemoryAssetStorage::new(Some(quota));
        let storage_context = StorageContext {
            room_id: RoomId::from_u128(0x12),
            namespace: BREAKOUT_MODULE_ID,
        };

        let content = b"some file content";

        let asset = stream::iter(vec![
            Ok(bytes::Bytes::from_static(content)),
            Err(AssetLoadError {
                source: Box::<dyn std::error::Error + Send + Sync>::from(anyhow!("stream error")),
            }),
        ]);
        let name = AssetMetaData {
            kind: asset_file_kind!("text"),
            timestamp: Timestamp::now(),
            // using pdf as extension here because this is the only extension
            // we currently have and it is not worth adding one only for tests
            extension: FileExtension::pdf(),
        };
        let uploaded = storage
            .upload_asset(asset.boxed(), name, &storage_context)
            .await;

        assert!(matches!(uploaded, Err(StorageError::ReadAsset(_))));
    }
}
