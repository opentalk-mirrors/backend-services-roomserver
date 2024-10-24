// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use crate::Router;
use axum::{
    async_trait,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::put,
    Json,
};
use opentalk_roomserver_types::room_parameters::RoomParameters;
use opentalk_types::api::error::ApiError;
use opentalk_types_common::rooms::RoomId;
use std::fmt::Debug;

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
            Self::Created => StatusCode::CREATED,
            Self::Updated => StatusCode::NO_CONTENT,
        }
        .into_response()
    }
}

#[async_trait]
pub trait RoomBackend: Clone + Send + Sync + Debug {
    /// Create or update the room.
    async fn put_room(
        &self,
        room_parameters: RoomParameters,
        room_id: RoomId,
    ) -> Result<RoomAction, ApiError>;
}

/// Creates a new room instance with the specified parameters.
///
/// If a room with the provided room ID already exists, the rooms idle timeout is refreshed.
#[tracing::instrument(level = "trace", skip(room_parameters), fields(room_id = %path.0))]
pub(crate) async fn put_room<B: RoomBackend>(
    State(ctx): State<B>,
    path: Path<RoomId>,
    Json(room_parameters): Json<RoomParameters>,
) -> Result<RoomAction, ApiError> {
    ctx.put_room(room_parameters, path.0).await
}

pub(crate) fn routes<B: RoomBackend + 'static>() -> Router<B> {
    Router::new().nest(
        "/rooms",
        Router::new().route("/:room_id", put(put_room::<B>)),
    )
}
