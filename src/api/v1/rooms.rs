// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use crate::api::Router;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json,
};
use opentalk_types::api::error::ApiError;

use crate::{api::Context, types::room_parameters::RoomParameters};
use axum_prometheus::metrics::counter;

/// Creates a new room instance with the specified parameters.
///
/// If a room with the provided room ID already exists, the rooms idle timeout is refreshed.
pub(crate) async fn post_create(
    State(ctx): State<Context>,
    Json(room_parameters): Json<RoomParameters>,
) -> Result<(), ApiError> {
    let room_id = room_parameters.room_id;

    let (created, task_handle) = ctx.room_tasks.create_room_if_not_exists(room_parameters);

    if created {
        return Ok(());
    }

    // Refresh the idle timeout if the room was not created with this request
    if let Err(err) = task_handle.refresh_idle_timeout().await {
        println!("Failed to refresh idle timeout for room {}: {err}", room_id);
        return Err(ApiError::internal());
    }

    Ok(())
}

#[tracing::instrument(level = "trace", skip(path), fields(room_id = %path.0))]
pub(crate) async fn probe_room(path: Path<String>) -> String {
    let room_id = path.0;

    log::trace!("Probing room {}", room_id);

    // Just an example for a custom metric (a counter in this case)
    counter!("probe_room_count_per_room", "room_id" => room_id.clone()).increment(1);

    format!("probing the room with id {}", room_id)
}

pub(crate) fn routes() -> Router {
    Router::new().nest(
        "/rooms",
        Router::new()
            .route("/create", post(post_create))
            .route("/probe/:room_id", get(probe_room)),
    )
}
