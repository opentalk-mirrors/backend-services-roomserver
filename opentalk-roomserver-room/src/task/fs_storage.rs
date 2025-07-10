// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    env, fs, io,
    path::PathBuf,
    sync::{
        Mutex, Once,
        atomic::{AtomicU64, Ordering},
    },
};

use anyhow::Context;
use async_trait::async_trait;
use opentalk_roomserver_signaling::{
    storage::{AssetMetaData, AssetUploaded, StorageError, StorageProvider, UploadResult},
    storage_quota::StorageQuota,
};
use opentalk_types_common::assets::AssetId;
use url::Url;

const DIR_NAME: &str = "opentalk-roomserver";

/// A simple storage provider using the local file system as storage backend.
///
/// This implementation is for testing purposes only. It will be moved to the
/// mocking module and only be used for tests once a real storage provider has
/// been implemented.
pub struct FsStorage {
    directory: PathBuf,
    quota: u64,
    used: AtomicU64,
    paths: Mutex<Vec<PathBuf>>,
}

impl FsStorage {
    /// Creates a new [`FsStorage`]
    ///
    /// * `quota` - The total size of files the user is allowed to upload (in bytes)
    /// * `directory` - The directory used to store the files. Defaults to an `opentalk-roomserver`
    ///   directory inside the temp directory when set to `None`.
    pub fn new(quota: u64, directory: Option<PathBuf>) -> Result<FsStorage, io::Error> {
        let directory = directory.unwrap_or_else(|| env::temp_dir().join(DIR_NAME));
        static CREATE_DIRECTORY: Once = Once::new();

        let mut result = None;
        CREATE_DIRECTORY.call_once(|| {
            if !directory.exists() {
                result = Some(fs::create_dir(&directory));
            }
        });

        if let Some(result) = result {
            result?;
        }

        Ok(FsStorage {
            directory,
            quota,
            used: AtomicU64::new(0),
            paths: Mutex::new(Vec::new()),
        })
    }
}

#[async_trait]
impl StorageProvider for FsStorage {
    async fn upload_file(&self, file: Vec<u8>, metadata: AssetMetaData) -> UploadResult {
        if self.used.load(Ordering::Relaxed) >= self.quota {
            return Err(StorageError::QuotaReached);
        }

        let id = AssetId::generate();
        let file_name = format!("{id}_{metadata}");
        let path = self.directory.join(file_name);

        let size = file.len() as u64;

        fs::write(&path, file).context("Writing file failed")?;

        self.used.fetch_add(size, Ordering::Relaxed);

        let url = format!("file://{}", path.to_string_lossy());
        self.paths.lock().unwrap().push(path);
        let url = Url::parse(&url).expect("Parsing url failed");

        Ok(AssetUploaded {
            id,
            remaining_quota: self.remaining_quota().await.into(),
            url,
        })
    }

    async fn remaining_quota(&self) -> StorageQuota {
        self.quota
            .saturating_sub(self.used.load(Ordering::Relaxed))
            .into()
    }
}

impl Drop for FsStorage {
    fn drop(&mut self) {
        // Delete all stored files
        for path in self.paths.lock().unwrap().drain(..) {
            if path.exists() && fs::remove_file(&path).is_err() {
                log::error!(
                    "Failed to remove stored file at '{}'",
                    path.to_string_lossy()
                );
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use opentalk_roomserver_signaling::storage::{AssetMetaData, StorageError, StorageProvider};
    use opentalk_types_common::{
        assets::{FileExtension, asset_file_kind},
        time::Timestamp,
    };

    use crate::task::fs_storage::FsStorage;

    #[tokio::test]
    async fn upload_file() {
        let quota = 5 * 1024u64.pow(3);
        let storage = FsStorage::new(quota, None).expect("Failed to create storage");

        let file = b"test".to_vec();
        let name = AssetMetaData {
            kind: asset_file_kind!("text"),
            timestamp: Timestamp::now(),
            // using pdf as extension here because this is the only extension
            // we currently have and it is not worth adding one only for tests
            extension: FileExtension::pdf(),
        };
        let uploaded = storage.upload_file(file.clone(), name).await.unwrap();
        let produced = fs::read(uploaded.url.path()).expect("File must exist");

        assert_eq!(file, produced);
    }

    #[tokio::test]
    async fn exceed_quota() {
        let quota = 1;
        let storage = FsStorage::new(quota, None).expect("Failed to create storage");

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
