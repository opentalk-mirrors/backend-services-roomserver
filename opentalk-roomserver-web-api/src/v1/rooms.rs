// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use crate::Router;
use axum::{
    async_trait,
    extract::{Path, State},
    routing::{get, put},
    Json,
};
use opentalk_roomserver_types::room_parameters::RoomParameters;
use opentalk_types::api::error::ApiError;
use opentalk_types_common::rooms::RoomId;
use std::fmt::Debug;

#[async_trait]
pub trait RoomBackend: Clone + Send + Sync + Debug {
    async fn create_room_if_not_exists(
        &self,
        room_parameters: RoomParameters,
        room_id: RoomId,
    ) -> Result<(), ApiError>;

    async fn probe_room(&self, path: Path<String>) -> String;
}

/// Creates a new room instance with the specified parameters.
///
/// If a room with the provided room ID already exists, the rooms idle timeout is refreshed.
#[tracing::instrument(level = "trace", skip(room_parameters), fields(room_id = %path.0))]
pub(crate) async fn put_room<B: RoomBackend>(
    State(ctx): State<B>,
    path: Path<RoomId>,
    Json(room_parameters): Json<RoomParameters>,
) -> Result<(), ApiError> {
    ctx.create_room_if_not_exists(room_parameters, path.0).await
}

#[tracing::instrument(level = "trace", skip(path), fields(room_id = %path.0))]
pub(crate) async fn probe_room<B: RoomBackend>(State(ctx): State<B>, path: Path<String>) -> String {
    ctx.probe_room(path).await
}

pub(crate) fn routes<B: RoomBackend + 'static>() -> Router<B> {
    Router::new().nest(
        "/rooms",
        Router::new()
            .route("/:room_id", put(put_room::<B>))
            .route("/probe/:room_id", get(probe_room::<B>)),
    )
}
