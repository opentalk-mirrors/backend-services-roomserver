// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::{HashMap, HashSet};

use anyhow::anyhow;
use async_trait::async_trait;
use opentalk_roomserver_signaling::storage::{
    AssetMetaData, AssetUploaded, StorageError, StorageProvider, UploadResult,
};
use opentalk_types_common::assets::AssetId;
use tokio::sync::Mutex;
use url::Url;

/// A simple storage provider using the local file system as storage backend.
///
/// This implementation is for testing purposes only. It will be moved to the
/// mocking module and only be used for tests once a real storage provider has
/// been implemented.
pub struct MemoryFileStorage {
    quota: u64,
    files: Mutex<HashMap<AssetId, Vec<u8>>>,
    running_uploads: Mutex<HashSet<AssetId>>,
}

impl MemoryFileStorage {
    /// Creates a new [`MemoryFileStorage`]
    ///
    /// * `quota` - The total size of files the user is allowed to upload (in bytes)
    pub fn new(quota: u64) -> Self {
        Self {
            quota,
            files: Mutex::new(HashMap::new()),
            running_uploads: Mutex::new(HashSet::new()),
        }
    }

    pub async fn file(&self, id: AssetId) -> Option<Vec<u8>> {
        self.files.lock().await.get(&id).cloned()
    }
}

#[async_trait]
impl StorageProvider for MemoryFileStorage {
    async fn upload_file(&self, file: Vec<u8>, metadata: AssetMetaData) -> UploadResult {
        if self.remaining_quota().await == 0 {
            return Err(StorageError::QuotaReached);
        }

        let id = AssetId::generate();
        self.files.lock().await.insert(id, file);

        Ok(AssetUploaded {
            id,
            filename: metadata.to_string(),
            remaining_quota: self.remaining_quota().await,
            url: self.file_url(id, &metadata),
        })
    }

    async fn upload_chunk(&self, id: AssetId, chunk: &[u8]) -> Result<(), StorageError> {
        let mut running_uploads = self.running_uploads.lock().await;
        if !running_uploads.contains(&id) {
            if self.remaining_quota().await == 0 {
                return Err(StorageError::QuotaReached);
            }
            running_uploads.insert(id);
        }

        let mut files = self.files.lock().await;
        let file = files.entry(id).or_default();
        file.extend_from_slice(chunk);

        Ok(())
    }

    async fn finalize_upload(&self, id: AssetId, metadata: AssetMetaData) -> UploadResult {
        if !self.running_uploads.lock().await.remove(&id) {
            return Err(anyhow!("No upload with id {id} running").into());
        }

        Ok(AssetUploaded {
            id,
            filename: metadata.to_string(),
            remaining_quota: self.remaining_quota().await,
            url: self.file_url(id, &metadata),
        })
    }

    async fn remaining_quota(&self) -> u64 {
        let used: usize = self.files.lock().await.values().map(Vec::len).sum();
        self.quota.saturating_sub(used as u64)
    }
}

impl MemoryFileStorage {
    fn file_url(&self, id: AssetId, metadata: &AssetMetaData) -> Url {
        let file_name = format!("{id}_{metadata}");
        let url = format!("file://{file_name}");
        Url::parse(&url).expect("Parsing url failed")
    }
}

#[cfg(test)]
mod test {
    use opentalk_roomserver_signaling::storage::{AssetMetaData, StorageError, StorageProvider};
    use opentalk_types_common::{
        assets::{FileExtension, asset_file_kind},
        time::Timestamp,
    };

    use crate::task::memory_file_storage::MemoryFileStorage;

    #[tokio::test]
    async fn upload_file() {
        let quota = 5 * 1024u64.pow(3);
        let storage = MemoryFileStorage::new(quota);

        let file = b"test".to_vec();
        let name = AssetMetaData {
            kind: asset_file_kind!("text"),
            timestamp: Timestamp::now(),
            // using pdf as extension here because this is the only extension
            // we currently have and it is not worth adding one only for tests
            extension: FileExtension::pdf(),
        };
        let uploaded = storage.upload_file(file.clone(), name).await.unwrap();
        let produced = storage.file(uploaded.id).await.unwrap();

        assert_eq!(file, produced);
    }

    #[tokio::test]
    async fn exceed_quota() {
        let quota = 1;
        let storage = MemoryFileStorage::new(quota);

        let file = b"file that exceeds the quota".to_vec();
        let name = AssetMetaData {
            kind: asset_file_kind!("text"),
            timestamp: Timestamp::now(),
            extension: FileExtension::pdf(),
        };
        storage.upload_file(file.clone(), name).await.unwrap();

        let name = AssetMetaData {
            kind: asset_file_kind!("text"),
            timestamp: Timestamp::now(),
            extension: FileExtension::pdf(),
        };
        let produced = storage.upload_file(file, name).await;

        assert!(matches!(produced, Err(StorageError::QuotaReached)));
    }
}
