// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use axum::{
    extract::Path,
    routing::{get, post},
};

use crate::api::Router;

pub(crate) async fn create_room() -> &'static str {
    "placeholder"
}

pub(crate) async fn probe_room(room_id: Path<String>) -> String {
    format!("probing the room with id {room_id:?}")
}

pub(crate) fn routes() -> Router {
    Router::new().nest(
        "/rooms",
        Router::new()
            .route("/create", post(create_room))
            .route("/probe/:room_id", get(probe_room)),
    )
}
