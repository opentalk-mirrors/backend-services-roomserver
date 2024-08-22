// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use crate::Router;
use axum::{
    async_trait,
    extract::{Path, State},
    routing::{get, post},
    Json,
};
use opentalk_roomserver_types::room_parameters::RoomParameters;
use opentalk_types::api::error::ApiError;
use std::fmt::Debug;

#[async_trait]
pub trait RoomContext: Clone + Send + Sync + Debug {
    async fn create_room_if_not_exists(
        &self,
        room_parameters: RoomParameters,
    ) -> Result<(), ApiError>;

    async fn probe_room(&self, path: Path<String>) -> String;
}

/// Creates a new room instance with the specified parameters.
///
/// If a room with the provided room ID already exists, the rooms idle timeout is refreshed.
pub(crate) async fn post_create<Api: RoomContext>(
    State(ctx): State<Api>,
    Json(room_parameters): Json<RoomParameters>,
) -> Result<(), ApiError> {
    ctx.create_room_if_not_exists(room_parameters).await
}

#[tracing::instrument(level = "trace", skip(path), fields(room_id = %path.0))]
pub(crate) async fn probe_room<Api: RoomContext>(
    State(ctx): State<Api>,
    path: Path<String>,
) -> String {
    ctx.probe_room(path).await
}

pub(crate) fn routes<Logic: RoomContext + 'static>() -> Router<Logic> {
    Router::new().nest(
        "/rooms",
        Router::new()
            .route("/create", post(post_create::<Logic>))
            .route("/probe/:room_id", get(probe_room::<Logic>)),
    )
}
