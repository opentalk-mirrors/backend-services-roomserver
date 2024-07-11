// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use crate::api::Router;
use axum::{
    extract::Path,
    routing::{get, post},
};
use axum_prometheus::metrics::counter;

pub(crate) async fn create_room() -> &'static str {
    "placeholder"
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
            .route("/create", post(create_room))
            .route("/probe/:room_id", get(probe_room)),
    )
}
