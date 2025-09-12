// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{fmt::Display, pin::Pin, sync::Arc};

use async_trait::async_trait;
use bytes::Bytes;
use futures::{Stream, StreamExt as _};
use opentalk_types_common::{
    assets::{AssetFileKind, AssetId, FileExtension},
    time::Timestamp,
};
use url::Url;

pub type UploadFuture<'a> = Pin<Box<dyn Future<Output = UploadResult> + Send + 'a>>;
pub type UploadResult = Result<AssetUploaded, StorageError>;

#[async_trait]
pub trait StorageProvider: Send + Sync {
    /// Uploads a file to the storage backend
    async fn upload_file(&self, file: Vec<u8>, metadata: AssetMetaData) -> UploadResult;

    /// Uploads a chunk of data to the storage backend
    async fn upload_chunk(&self, id: AssetId, chunk: &[u8]) -> Result<(), StorageError>;

    /// Finalizes an upload of one or multiple chunks
    async fn finalize_upload(&self, id: AssetId, metadata: AssetMetaData) -> UploadResult;

    async fn remaining_quota(&self) -> u64;
}

pub async fn upload_stream<E>(
    storage_client: Arc<dyn StorageProvider>,
    stream: &mut (impl Stream<Item = Result<Bytes, E>> + Unpin + Sized),
    metadata: AssetMetaData,
) -> Result<AssetUploaded, StorageError>
where
    StorageError: From<E>,
{
    let id = AssetId::generate();
    while let Some(bytes) = stream.next().await {
        storage_client.upload_chunk(id, &bytes?).await?;
    }

    storage_client.finalize_upload(id, metadata).await
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

impl From<reqwest::Error> for StorageError {
    fn from(err: reqwest::Error) -> Self {
        StorageError::StorageError(anyhow::Error::new(err))
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
#[derive(Debug)]
pub struct AssetUploaded {
    pub id: AssetId,
    pub filename: String,
    pub remaining_quota: u64,
    pub url: Url,
}
