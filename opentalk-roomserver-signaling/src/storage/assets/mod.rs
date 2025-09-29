// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! Storage module for handling asset uploads in the RoomServer.
//!
//! This module provides the [`ModuleAssetStorage`] struct, which is used by signaling modules to
//! store assets. The [`ModuleAssetStorage`] acts as a wrapper around a storage provider
//! implementation, adding contextual information like the room ID and module namespace to each
//! operation.
//!
//! The [`AssetStorageProvider`] trait defines the interface for storage backends, allowing
//! different implementations (e.g., local filesystem, cloud storage) to be plugged in as needed.
//! All asset uploads, chunked uploads, and quota management are handled through this trait.

pub mod provider;

use std::{
    fmt::{Debug, Display},
    pin::Pin,
    sync::Arc,
};

use bytes::Bytes;
use futures::{Stream, StreamExt as _};
use opentalk_types_common::{
    assets::{AssetFileKind, AssetId, FileExtension},
    time::Timestamp,
};
use url::Url;

use crate::storage::{StorageContext, assets::provider::AssetStorageProvider};

pub type UploadFuture<'a> = Pin<Box<dyn Future<Output = UploadResult> + Send + 'a>>;
pub type UploadResult = Result<AssetUploaded, StorageError>;

/// Provides storage operations for signaling modules, wrapping a [`AssetStorageProvider`] with
/// contextual information such as room ID and module namespace.
#[derive(Debug, Clone)]
pub struct ModuleAssetStorage {
    provider: Arc<dyn AssetStorageProvider>,
    context: StorageContext,
}

impl ModuleAssetStorage {
    pub fn new(provider: Arc<dyn AssetStorageProvider>, context: StorageContext) -> Self {
        Self { provider, context }
    }

    /// Uploads an asset to the storage backend
    pub async fn upload_asset(&self, asset: Vec<u8>, metadata: AssetMetaData) -> UploadResult {
        self.provider
            .upload_asset(asset, metadata, &self.context)
            .await
    }

    /// Uploads a chunk of data to the storage backend
    pub async fn upload_chunk(&self, id: AssetId, chunk: &[u8]) -> Result<(), StorageError> {
        self.provider.upload_chunk(id, chunk, &self.context).await
    }

    /// Finalizes an upload of one or multiple chunks
    pub async fn finalize_upload(&self, id: AssetId, metadata: AssetMetaData) -> UploadResult {
        self.provider
            .finalize_upload(id, metadata, &self.context)
            .await
    }

    pub async fn remaining_quota(&self) -> Option<u64> {
        self.provider.remaining_quota(&self.context).await
    }

    pub async fn upload_stream<E>(
        self,
        stream: &mut (impl Stream<Item = Result<Bytes, E>> + Unpin + Sized),
        metadata: AssetMetaData,
    ) -> Result<AssetUploaded, StorageError>
    where
        StorageError: From<E>,
    {
        let id = AssetId::generate();
        while let Some(bytes) = stream.next().await {
            self.upload_chunk(id, &bytes?).await?;
        }

        self.finalize_upload(id, metadata).await
    }
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
    pub remaining_quota: Option<u64>,
    pub url: Url,
}
