// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{fmt::Display, pin::Pin};

use async_trait::async_trait;
use opentalk_types_common::{
    assets::{AssetFileKind, AssetId, FileExtension},
    time::Timestamp,
};
use url::Url;

use crate::storage_quota::StorageQuota;

pub type UploadFuture<'a> = Pin<Box<dyn Future<Output = UploadResult> + Send + 'a>>;
pub type UploadResult = Result<AssetUploaded, StorageError>;
pub type QuotaFuture<'a> = Pin<Box<dyn Future<Output = StorageQuota> + Send + 'a>>;

#[async_trait]
pub trait StorageProvider: Send + Sync {
    /// Uploads a file to the storage backend
    async fn upload_file(&self, file: Vec<u8>, metadata: AssetMetaData) -> UploadResult;

    async fn remaining_quota(&self) -> StorageQuota;
}

#[derive(Debug)]
pub enum StorageError {
    /// The quota was reached before the current upload
    QuotaReached,
    StorageError(anyhow::Error),
}

impl From<anyhow::Error> for StorageError {
    fn from(err: anyhow::Error) -> Self {
        StorageError::StorageError(err)
    }
}

/// Metadata about an stored asset
pub struct AssetMetaData {
    /// The kind of the asset
    pub kind: AssetFileKind,
    /// The timestamp of the upload
    pub timestamp: Timestamp,
    /// The filename extension
    pub extension: FileExtension,
}

impl Display for AssetMetaData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}_{}{}",
            self.kind,
            self.timestamp.to_string_for_filename(),
            self.extension.to_string_with_leading_dot()
        )
    }
}

/// Information about an asset that has been uploaded to a storage backend
pub struct AssetUploaded {
    pub id: AssetId,
    pub filename: String,
    pub remaining_quota: u64,
    pub url: Url,
}
