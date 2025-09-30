// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::sync::Arc;

use anyhow::Context as _;
use async_trait::async_trait;
use opentalk_roomserver_signaling::storage::{
    StorageContext,
    assets::{
        AssetMetaData, AssetUploaded, StorageError, UploadResult,
        provider::{AssetStorageProvider, AssetStream},
    },
};
use opentalk_types_api_v1::{error::ApiError, services::roomserver::PostAssetResponseBody};
use reqwest::{Body, header::CONTENT_TYPE};
use tokio::sync::RwLock;
use url::Url;

const MAX_BODY_PREVIEW_LENGTH: usize = 80;

/// `ControllerAssetStorage` is an implementation of the [`AssetStorageProvider`] trait that
/// interacts with a remote controller service to manage asset uploads and storage quota for a
/// roomserver.
#[derive(Debug)]
pub struct ControllerAssetStorage {
    base_url: Url,
    secret: String,
    client: reqwest::Client,
    quota: RwLock<Option<u64>>,
}

impl ControllerAssetStorage {
    pub fn new(base_url: Url, secret: String, quota: Option<u64>) -> Self {
        Self {
            base_url,
            secret,
            client: reqwest::Client::new(),
            quota: RwLock::new(quota),
        }
    }
}

#[async_trait]
impl AssetStorageProvider for ControllerAssetStorage {
    async fn upload_asset(
        &self,
        asset: AssetStream,
        metadata: AssetMetaData,
        context: &StorageContext,
    ) -> UploadResult {
        let mut query = vec![
            ("namespace", context.namespace.as_str()),
            ("file_extension", metadata.extension.as_str()),
            ("kind", metadata.kind.as_str()),
        ];
        if let Some(event_title) = context.event.as_ref().map(|e| e.title.as_str()) {
            query.push(("event_title", event_title));
        }
        let response = self
            .client
            .post(
                self.base_url
                    .join(&format!(
                        "/v1/services/roomserver/room/{}/asset",
                        context.room_id
                    ))
                    .context("Invalid URL")?,
            )
            .bearer_auth(&self.secret)
            .body(Body::wrap_stream(asset))
            .header(CONTENT_TYPE, "application/octet-stream")
            .query(&query)
            .send()
            .await
            .context("Asset upload request failed")?;

        let status = response.status();
        let body = response
            .bytes()
            .await
            .context("Failed to read upload asset response body")?;

        if status == ApiError::storage_quota_exceeded().status {
            tracing::debug!("Asset upload failed due to exceeded storage quota");
            return Err(StorageError::QuotaReached);
        }
        if !status.is_success() {
            let body_str = match std::str::from_utf8(&body) {
                Ok(s) if s.len() > MAX_BODY_PREVIEW_LENGTH => {
                    format!("{}...[truncated]", &s[..MAX_BODY_PREVIEW_LENGTH])
                }
                Ok(s) => s.to_string(),
                Err(_) => "<non-utf8>".to_string(),
            };
            tracing::error!(
                "Asset upload failed with status: `{}` Body: `{}`",
                status,
                body_str,
            );
            return Err(
                anyhow::anyhow!("Failed to upload asset due to unexpected server error").into(),
            );
        }

        let PostAssetResponseBody {
            asset_resource,
            remaining_quota_bytes,
        } = serde_json::from_slice(&body)
            .inspect_err(|e| {
                tracing::error!(
                    "Failed to parse upload response: {e:?} Body: {:?}",
                    std::str::from_utf8(&body).unwrap_or("<non-utf8>")
                );
            })
            .context("Failed to parse upload asset response body")?;

        if let Some(remaining_quota_bytes) = remaining_quota_bytes {
            self.quota.write().await.replace(remaining_quota_bytes);
        }

        Ok(AssetUploaded {
            id: asset_resource.id,
            filename: asset_resource.filename,
            remaining_quota: remaining_quota_bytes,
        })
    }

    async fn remaining_quota(&self, _context: &StorageContext) -> Option<u64> {
        *self.quota.read().await
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn std::any::Any + Send + Sync> {
        self
    }
}
