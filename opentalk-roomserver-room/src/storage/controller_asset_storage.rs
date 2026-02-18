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
use opentalk_service_auth::ApiKey;
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
    secret: ApiKey,
    client: reqwest::Client,
    quota: RwLock<Option<u64>>,
}

impl ControllerAssetStorage {
    pub fn new(base_url: Url, secret: ApiKey, quota: Option<u64>) -> Self {
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

        let token = self
            .secret
            .generate_jwt()
            .context("Failed to generate auth header")?;

        let response = self
            .client
            .post(
                self.base_url
                    .join(&format!("/internal/room/{}/asset", context.room_id))
                    .context("Invalid URL")?,
            )
            .bearer_auth(token)
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
            return Err(StorageError::QuotaExceeded);
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

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use bytes::Bytes;
    use futures::{StreamExt, stream};
    use mockito::Matcher;
    use opentalk_roomserver_signaling::storage::{
        StorageContext,
        assets::{
            AssetMetaData, AssetUploaded, StorageError,
            provider::{AssetLoadError, AssetStorageProvider},
        },
    };
    use opentalk_service_auth::ApiKey;
    use opentalk_types_api_v1::{
        assets::AssetResource, error::ApiError, services::roomserver::PostAssetResponseBody,
    };
    use opentalk_types_common::{
        assets::{AssetFileKind, AssetId, FileExtension},
        modules::ModuleId,
        rooms::RoomId,
        time::Timestamp,
    };
    use pretty_assertions::assert_eq;

    use super::ControllerAssetStorage;

    #[test_log::test(tokio::test)]
    async fn successful_asset_upload() {
        let room_id = RoomId::from_u128(0x01);
        let asset_file_kind = AssetFileKind::example_data();
        let created_at = Timestamp::unix_epoch();
        let module_id = ModuleId::example_data();
        let response_body = PostAssetResponseBody {
            asset_resource: AssetResource {
                id: AssetId::from_u128(0x01),
                filename: "Testfile".to_string(),
                namespace: Some(module_id.clone()),
                created_at: *created_at,
                kind: asset_file_kind.to_string(),
                size: 12.into(),
            },
            remaining_quota_bytes: None,
        };

        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", format!("/internal/room/{room_id}/asset").as_str())
            .match_header("authorization", Matcher::Regex("^Bearer .*".into()))
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("namespace".to_string(), module_id.to_string()),
                mockito::Matcher::UrlEncoded(
                    "file_extension".to_string(),
                    FileExtension::pdf().to_string(),
                ),
                mockito::Matcher::UrlEncoded("kind".to_string(), asset_file_kind.to_string()),
            ]))
            // response
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_body(serde_json::to_string(&response_body).unwrap())
            .create_async()
            .await;

        let provider = ControllerAssetStorage::new(
            server.url().parse().unwrap(),
            ApiKey::new("controller", "secret"),
            None,
        );
        let asset_stream =
            stream::once(async { Ok(Bytes::copy_from_slice(r"asdasd".as_bytes())) }).boxed();

        let upload_result = provider
            .upload_asset(
                asset_stream,
                AssetMetaData {
                    kind: asset_file_kind,
                    timestamp: created_at,
                    extension: FileExtension::pdf(),
                },
                &StorageContext {
                    room_id,
                    namespace: ModuleId::example_data(),
                    event: None,
                },
            )
            .await;

        mock.assert_async().await;

        assert_eq!(
            upload_result.unwrap(),
            AssetUploaded {
                id: response_body.asset_resource.id,
                filename: response_body.asset_resource.filename,
                remaining_quota: response_body.remaining_quota_bytes
            }
        );
    }

    #[test_log::test(tokio::test)]
    async fn quota_exceeded_asset_upload() {
        let room_id = RoomId::from_u128(0x01);
        let asset_file_kind = AssetFileKind::example_data();
        let created_at = Timestamp::unix_epoch();
        let module_id = ModuleId::example_data();

        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", format!("/internal/room/{room_id}/asset").as_str())
            .match_header("authorization", Matcher::Regex("^Bearer .*".into()))
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("namespace".to_string(), module_id.to_string()),
                mockito::Matcher::UrlEncoded(
                    "file_extension".to_string(),
                    FileExtension::pdf().to_string(),
                ),
                mockito::Matcher::UrlEncoded("kind".to_string(), asset_file_kind.to_string()),
            ]))
            // response
            .with_status(ApiError::storage_quota_exceeded().status.as_u16() as usize)
            .create_async()
            .await;

        let provider = ControllerAssetStorage::new(
            server.url().parse().unwrap(),
            ApiKey::new("controller", "secret"),
            None,
        );
        let asset_stream =
            stream::once(async { Ok(Bytes::copy_from_slice(r"asdasd".as_bytes())) }).boxed();

        let upload_result = provider
            .upload_asset(
                asset_stream,
                AssetMetaData {
                    kind: asset_file_kind,
                    timestamp: created_at,
                    extension: FileExtension::pdf(),
                },
                &StorageContext {
                    room_id,
                    namespace: ModuleId::example_data(),
                    event: None,
                },
            )
            .await;

        mock.assert_async().await;

        assert!(matches!(upload_result, Err(StorageError::QuotaExceeded)));
    }

    #[test_log::test(tokio::test)]
    async fn test_asset_load_error() {
        let room_id = RoomId::from_u128(0x01);
        let asset_file_kind = AssetFileKind::example_data();
        let created_at = Timestamp::unix_epoch();
        let module_id = ModuleId::example_data();

        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", format!("/internal/room/{room_id}/asset").as_str())
            .match_header("authorization", Matcher::Regex("^Bearer .*".into()))
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("namespace".to_string(), module_id.to_string()),
                mockito::Matcher::UrlEncoded(
                    "file_extension".to_string(),
                    FileExtension::pdf().to_string(),
                ),
                mockito::Matcher::UrlEncoded("kind".to_string(), asset_file_kind.to_string()),
            ]))
            // response
            .with_status(ApiError::storage_quota_exceeded().status.as_u16() as usize)
            .create_async()
            .await;

        let provider = ControllerAssetStorage::new(
            server.url().parse().unwrap(),
            ApiKey::new("controller", "secret"),
            None,
        );

        let asset_stream = stream::iter(vec![Err(AssetLoadError {
            source: anyhow!("Error").into(),
        })])
        .boxed();
        let upload_result = provider
            .upload_asset(
                asset_stream,
                AssetMetaData {
                    kind: asset_file_kind,
                    timestamp: created_at,
                    extension: FileExtension::pdf(),
                },
                &StorageContext {
                    room_id,
                    namespace: ModuleId::example_data(),
                    event: None,
                },
            )
            .await;

        assert!(matches!(upload_result, Err(StorageError::Internal(_))));
    }
}
