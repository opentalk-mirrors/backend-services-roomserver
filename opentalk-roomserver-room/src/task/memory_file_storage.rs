// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use anyhow::anyhow;
use async_trait::async_trait;
use opentalk_roomserver_signaling::storage::{
    AssetMetaData, AssetUploaded, StorageContext, StorageError, UploadResult,
    provider::AssetStorageProvider,
};
use opentalk_types_common::assets::AssetId;
use tokio::sync::Mutex;
use url::Url;

/// A simple storage provider using the local file system as storage backend.
///
/// This implementation is for testing purposes only. It will be moved to the
/// mocking module and only be used for tests once a real storage provider has
/// been implemented.
#[derive(Debug)]
pub struct MemoryAssetStorage {
    quota: Option<u64>,
    files: Mutex<HashMap<AssetId, Vec<u8>>>,
    running_uploads: Mutex<HashSet<AssetId>>,
}

impl MemoryAssetStorage {
    /// Creates a new [`MemoryAssetStorage`]
    ///
    /// * `quota` - The total size of files the user is allowed to upload (in bytes)
    pub fn new(quota: Option<u64>) -> Self {
        Self {
            quota,
            files: Mutex::new(HashMap::new()),
            running_uploads: Mutex::new(HashSet::new()),
        }
    }

    pub async fn file(&self, id: AssetId) -> Option<Vec<u8>> {
        self.files.lock().await.get(&id).cloned()
    }

    pub async fn file_count(&self) -> usize {
        self.files.lock().await.len()
    }
}

#[async_trait]
impl AssetStorageProvider for MemoryAssetStorage {
    async fn upload_file(
        &self,
        file: Vec<u8>,
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

        let id = AssetId::generate();
        self.files.lock().await.insert(id, file);

        Ok(AssetUploaded {
            id,
            filename: metadata.to_string(),
            remaining_quota: self.remaining_quota(context).await,
            url: Self::file_url(id, &metadata),
        })
    }

    async fn upload_chunk(
        &self,
        id: AssetId,
        chunk: &[u8],
        context: &StorageContext,
    ) -> Result<(), StorageError> {
        let mut running_uploads = self.running_uploads.lock().await;
        if !running_uploads.contains(&id) {
            if self
                .remaining_quota(context)
                .await
                .map(|q| q == 0)
                .unwrap_or(false)
            {
                return Err(StorageError::QuotaReached);
            }
            running_uploads.insert(id);
        }

        let mut files = self.files.lock().await;
        let file = files.entry(id).or_default();
        file.extend_from_slice(chunk);

        Ok(())
    }

    async fn finalize_upload(
        &self,
        id: AssetId,
        metadata: AssetMetaData,
        context: &StorageContext,
    ) -> UploadResult {
        if !self.running_uploads.lock().await.remove(&id) {
            return Err(anyhow!("No upload with id {id} running").into());
        }

        Ok(AssetUploaded {
            id,
            filename: metadata.to_string(),
            remaining_quota: self.remaining_quota(context).await,
            url: Self::file_url(id, &metadata),
        })
    }

    async fn remaining_quota(&self, _context: &StorageContext) -> Option<u64> {
        if let Some(q) = self.quota {
            let used: usize = self.files.lock().await.values().map(Vec::len).sum();
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
    fn file_url(id: AssetId, metadata: &AssetMetaData) -> Url {
        let file_name = format!("{id}_{metadata}");
        let url = format!("file://{file_name}");
        Url::parse(&url).expect("Parsing url failed")
    }
}

#[cfg(test)]
mod test {
    use opentalk_roomserver_signaling::storage::{
        AssetMetaData, StorageContext, StorageError, provider::AssetStorageProvider as _,
    };
    use opentalk_roomserver_types::breakout::BREAKOUT_MODULE_ID;
    use opentalk_types_common::{
        assets::{FileExtension, asset_file_kind},
        rooms::RoomId,
        time::Timestamp,
    };

    use crate::task::memory_file_storage::MemoryAssetStorage;

    #[tokio::test]
    async fn upload_file() {
        let quota = 5 * 1024u64.pow(3);
        let storage = MemoryAssetStorage::new(Some(quota));
        let storage_context = StorageContext {
            room_id: RoomId::from_u128(0x12),
            namespace: BREAKOUT_MODULE_ID,
        };

        let file = b"test".to_vec();
        let name = AssetMetaData {
            kind: asset_file_kind!("text"),
            timestamp: Timestamp::now(),
            // using pdf as extension here because this is the only extension
            // we currently have and it is not worth adding one only for tests
            extension: FileExtension::pdf(),
        };
        let uploaded = storage
            .upload_file(file.clone(), name, &storage_context)
            .await
            .unwrap();
        let produced = storage.file(uploaded.id).await.unwrap();

        assert_eq!(file, produced);
    }

    #[tokio::test]
    async fn exceed_quota() {
        let quota = 1;
        let storage = MemoryAssetStorage::new(Some(quota));
        let storage_context = StorageContext {
            room_id: RoomId::from_u128(0x12),
            namespace: BREAKOUT_MODULE_ID,
        };

        let file = b"file that exceeds the quota".to_vec();
        let name = AssetMetaData {
            kind: asset_file_kind!("text"),
            timestamp: Timestamp::now(),
            extension: FileExtension::pdf(),
        };
        storage
            .upload_file(file.clone(), name, &storage_context)
            .await
            .unwrap();

        let name = AssetMetaData {
            kind: asset_file_kind!("text"),
            timestamp: Timestamp::now(),
            extension: FileExtension::pdf(),
        };
        let produced = storage.upload_file(file, name, &storage_context).await;

        assert!(matches!(produced, Err(StorageError::QuotaReached)));
    }
}
