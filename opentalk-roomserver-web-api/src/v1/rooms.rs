// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::fmt::Debug;

use async_trait::async_trait;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{post, put},
};
use opentalk_roomserver_types::{
    api::{TokenRequestBody, TokenResponse},
    client_parameters::ClientParameters,
    room_parameters::RoomParameters,
};
use opentalk_types_api_v1::error::ApiError;
use opentalk_types_common::{rooms::RoomId, roomserver::Token};

use crate::Router;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RoomAction {
    Created,
    Updated,
}

impl RoomAction {
    /// Returns `true` if the room action is [`Created`].
    ///
    /// [`Created`]: RoomAction::Created
    #[must_use]
    pub fn is_created(&self) -> bool {
        matches!(self, Self::Created)
    }
}

impl IntoResponse for RoomAction {
    fn into_response(self) -> Response {
        match self {
            Self::Created => StatusCode::CREATED.into_response(),
            Self::Updated => StatusCode::NO_CONTENT.into_response(),
        }
    }
}

#[async_trait]
pub trait RoomBackend: Clone + Send + Sync + Debug {
    /// Create or update the room.
    async fn put_room(
        &self,
        room_id: RoomId,
        room_parameters: RoomParameters,
    ) -> Result<RoomAction, ApiError>;

    async fn request_room_token(
        &mut self,
        room_id: RoomId,
        client_parameters: ClientParameters,
        room_parameters: Option<RoomParameters>,
    ) -> Result<Option<Token>, ApiError>;
}

/// Creates a new room instance with the specified parameters if no room with the provided id
/// exists.
///
/// If a room with the provided room ID already exists, the rooms idle timeout is refreshed.
#[utoipa::path(
    put,
    path = "/rooms/{room_id}",
    request_body = RoomParameters,
    params(
        ("room_id" = RoomId, Path, description = "The UUID that identifies the room")
    ),
    responses(
        (status = StatusCode::CREATED, description = "Successfully created a new room"),
        (status = StatusCode::NO_CONTENT, description = "The room did exist before and the parameter were updated if necessary"),
        (status = StatusCode::UNAUTHORIZED, description = "The provided API token is invalid"),
        (status = StatusCode::BAD_REQUEST, description = "The provided API token could not be parsed"),
        (status = StatusCode::UNPROCESSABLE_ENTITY, description = "Failed to parse request body"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "An internal server error occurred"),
    ),
    security(
        ("API-Token" = [])
    )
    )]
#[tracing::instrument(level = "info", name = "/rooms/{room_id}", skip_all, fields(opentalk.room_id = %path.0))]
pub(crate) async fn put_room<B: RoomBackend>(
    State(ctx): State<B>,
    path: Path<RoomId>,
    Json(room_parameters): Json<RoomParameters>,
) -> Result<RoomAction, ApiError> {
    ctx.put_room(path.0, room_parameters).await
}

/// Creates a new signaling token for the specified user and room
///
/// The signaling token can be used to establish a websocket connection with the roomserver through
/// the `/signaling/<token>` endpoint. The token has a limited lifetime (30 seconds by default) and
/// can only be used once.
///
/// Calling this endpoint will start a new room task or refresh existing ones. To get a token for an
/// unknown room, the request body has to contain the `room_parameters` field (See
/// [`TokenRequestBody`]). If the room is already running, any provided `room_parameters` will be
/// ignored.
#[utoipa::path(
    post,
    path = "/rooms/{room_id}/token",
    request_body = TokenRequestBody,
    params(
        ("room_id" = RoomId, Path, description = "The UUID that identifies the room")
    ),
    responses(
        (status = StatusCode::OK, description = "The response body contains either the signaling token, or a prompt to include the associated room parameters in the request", body = TokenResponse),
        (status = StatusCode::UNAUTHORIZED, description = "The provided API token is invalid"),
        (status = StatusCode::BAD_REQUEST, description = "The provided API token could not be parsed"),
        (status = StatusCode::UNPROCESSABLE_ENTITY, description = "Failed to parse request body"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "An internal server error occurred"),
    ),
    security(
        ("API-Token" = [])
    )
    )]
#[tracing::instrument(level = "info", name = "/rooms/{room_id}/token", skip_all, fields(opentalk.room_id = %path.0, http.method = "PUT"))]
pub(crate) async fn request_token<B: RoomBackend>(
    State(mut ctx): State<B>,
    path: Path<RoomId>,
    Json(body): Json<TokenRequestBody>,
) -> Result<Json<TokenResponse>, ApiError> {
    let response = match ctx
        .request_room_token(path.0, body.client_parameters, body.room_parameters)
        .await?
    {
        Some(token) => TokenResponse::Token { token },
        None => TokenResponse::UnknownRoom,
    };

    Ok(Json(response))
}

pub(crate) fn routes<B: RoomBackend + 'static>() -> Router<B> {
    Router::new().nest(
        "/rooms",
        Router::new()
            .route("/{room_id}", put(put_room::<B>))
            .route("/{room_id}/token", post(request_token::<B>)),
    )
}
