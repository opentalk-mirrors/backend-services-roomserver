// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::str::FromStr;

use async_trait::async_trait;
use axum::{
    extract::{Query, RawQuery, State, WebSocketUpgrade},
    http::{HeaderMap, header::AUTHORIZATION},
    response::Response,
    routing::{get, post},
};
use livekit_api::access_token::Claims;
pub use opentalk_roomserver_types::livekit_proxy::{
    LiveKitProxyRequest, LiveKitProxyTarget, PreparedSocket,
};
use opentalk_roomserver_types::{
    LIVEKIT_SUBROOM_AUDIO_ROOM_DELIMITER, breakout::breakout_id::BreakoutId,
    connection_id::ConnectionId, livekit_proxy::adapter::LiveKitSocketAdapter, room_kind::RoomKind,
};
use opentalk_types_api_internal::error::ApiError;
use opentalk_types_common::rooms::RoomId;
use opentalk_types_signaling::ParticipantId;
use serde::Deserialize;
use uuid::Uuid;

use crate::Router;

pub fn routes<B: LiveKitProxyBackend + 'static>() -> Router<B> {
    Router::new().nest(
        "/livekit/rtc",
        Router::new()
            .route("/", get(proxy_socket::<B>))
            .route("/v1", get(proxy_socket::<B>))
            .route("/validate", post(proxy_validate::<B>))
            .route("/v1/validate", post(proxy_validate::<B>)),
    )
}

#[async_trait]
pub trait LiveKitProxyBackend: Clone + Send + Sync + std::fmt::Debug {
    async fn connect_upstream_socket(
        &self,
        ws_request: LiveKitProxyRequest,
    ) -> Result<PreparedSocket, ApiError>;

    async fn connect_downstream_socket(
        &self,
        ws_request: LiveKitProxyRequest,
        upstream_socket: PreparedSocket,
        socket: LiveKitSocketAdapter,
    ) -> Result<(), ApiError>;

    /// Proxies a LiveKit REST validation request to the livekit module
    async fn proxy_livekit_validate(
        &self,
        room_id: RoomId,
        headers: HeaderMap,
        raw_query: Option<String>,
    ) -> Result<Response, ApiError>;
}

#[derive(Debug, Deserialize)]
pub(crate) struct LiveKitQuery {
    access_token: Option<String>,
}

/// Opens a new LiveKit WebSocket connection for RTC communication.
///
/// This endpoint establishes a WebSocket connection to the upstream LiveKit server.
/// The access token can be provided either as a query parameter or in the `Authorization`
/// header with the `Bearer` scheme.
///
/// # Available paths
/// - `/livekit/rtc`
/// - `/livekit/rtc/v1`
#[utoipa::path(
    get,
    path = "/livekit/rtc",
    responses(
        (status = StatusCode::SWITCHING_PROTOCOLS, description = "Successfully upgraded connection to WebSocket for RTC communication"),
        (status = StatusCode::BAD_REQUEST, description = "Invalid token format or missing required token claims"),
        (status = StatusCode::UNAUTHORIZED, description = "Missing access token or invalid token scheme"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "An internal server error occurred"),
    ),
    params(
        ("access_token" = Option<String>, Query, description = "LiveKit access token (JWT) as query parameter. If not provided, must be in Authorization header with Bearer scheme")
    ),
    security(
        ("Livekit-Token" = [])
    ),
)]
#[tracing::instrument(level = "info", name = "/livekit/proxy/rtc", skip_all)]
pub(crate) async fn proxy_socket<B: LiveKitProxyBackend + 'static>(
    State(ctx): State<B>,
    RawQuery(raw_query): RawQuery,
    Query(query): Query<LiveKitQuery>,
    ws_upgrade: WebSocketUpgrade,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let access_token = extract_access_token(query, &headers)?;

    // we do not verify the token since this is done by livekit. We only proxy the connection.
    let content = jsonwebtoken::dangerous::insecure_decode::<Claims>(access_token.as_bytes())
        .map_err(|_| ApiError::bad_request())?;
    let (room_id, proxy_target) = parse_livekit_room_id(&content.claims.video.room)?;
    let (participant_id, connection_id) = parse_livekit_participant(&content.claims.sub)?;

    let websocket_request = LiveKitProxyRequest {
        raw_query,
        headers,
        room_id,
        proxy_target,
        participant_id,
        connection_id,
    };

    let upstream_socket = ctx
        .connect_upstream_socket(websocket_request.clone())
        .await?;

    let backend = ctx.clone();
    Ok(ws_upgrade.on_upgrade(move |websocket| async move {
        let socket = LiveKitSocketAdapter::new(websocket);
        if let Err(err) = backend
            .connect_downstream_socket(websocket_request, upstream_socket, socket)
            .await
        {
            tracing::warn!("failed to accept livekit websocket: {err:?}");
        }
    }))
}

/// Proxies the LiveKit validate request to the upstream livekit service via the room task
///
/// # Available paths
/// - `/livekit/rtc/validate`
/// - `/livekit/rtc/v1/validate`
#[utoipa::path(
    post,
    path = "/livekit/rtc/validate",
    responses(
        (status = StatusCode::OK, description = "Validation response from LiveKit"),
        (status = StatusCode::UNPROCESSABLE_ENTITY, description = "No livekit module configured for the request"),
        (status = StatusCode::BAD_REQUEST, description = "Invalid request headers"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "An internal server error occurred"),
    ),
    security(
        ("Livekit-Token" = [])
    ),
)]
#[tracing::instrument(level = "info", name = "/livekit/rtc/validate", skip_all)]
pub(crate) async fn proxy_validate<B: LiveKitProxyBackend>(
    State(ctx): State<B>,
    RawQuery(raw_query): RawQuery,
    Query(query): Query<LiveKitQuery>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let access_token = extract_access_token(query, &headers)?;
    let content = jsonwebtoken::dangerous::insecure_decode::<Claims>(access_token.as_bytes())
        .map_err(|_| ApiError::bad_request())?;

    let (room_id, _) = parse_livekit_room_id(&content.claims.video.room)?;

    ctx.proxy_livekit_validate(room_id, headers, raw_query)
        .await
}

fn parse_livekit_room_id(livekit_id: &str) -> Result<(RoomId, LiveKitProxyTarget), ApiError> {
    if let Some((room_id, whisper_id)) = livekit_id.split_once(LIVEKIT_SUBROOM_AUDIO_ROOM_DELIMITER)
    {
        let room_id = RoomId::from_str(room_id).map_err(|_| ApiError::bad_request())?;
        let whisper_id = Uuid::from_str(whisper_id).map_err(|_| ApiError::bad_request())?;

        return Ok((room_id, LiveKitProxyTarget::SubroomAudio { whisper_id }));
    }

    let (room_id, room_kind) = livekit_id
        .split_once(':')
        .ok_or_else(ApiError::bad_request)?;

    let room_id = RoomId::from_str(room_id).map_err(|_| ApiError::bad_request())?;
    let room_kind = if room_kind == "main" {
        RoomKind::Main
    } else {
        RoomKind::Breakout(BreakoutId::from_str(room_kind).map_err(|_| ApiError::bad_request())?)
    };

    Ok((room_id, LiveKitProxyTarget::LiveKit { room_kind }))
}

fn parse_livekit_participant(livekit_sub: &str) -> Result<(ParticipantId, ConnectionId), ApiError> {
    let Some((participant, connection)) = livekit_sub.split_once(':') else {
        return Err(ApiError::bad_request());
    };

    let participant = participant.parse().map_err(|_| ApiError::bad_request())?;
    let connection = connection.parse().map_err(|_| ApiError::bad_request())?;

    Ok((participant, connection))
}

fn extract_access_token(query: LiveKitQuery, headers: &HeaderMap) -> Result<String, ApiError> {
    if let Some(token) = query.access_token {
        Ok(token)
    } else {
        let Some(access_token) = headers
            .get(&AUTHORIZATION)
            .and_then(|header_value| header_value.to_str().ok())
            .and_then(|header_value| {
                let (scheme, token) = header_value.split_once(' ')?;

                if scheme.eq_ignore_ascii_case("bearer") {
                    Some(token.to_string())
                } else {
                    None
                }
            })
        else {
            return Err(ApiError::unauthorized());
        };

        Ok(access_token)
    }
}
