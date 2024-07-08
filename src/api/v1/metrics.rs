// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use axum::routing::get;

use crate::api::{Context, Router};
use axum::extract::State;

pub(crate) async fn metrics(_context: State<Context>) -> &'static str {
    "very cool metrics"
}

pub(crate) fn routes() -> Router {
    Router::new().route("/metrics", get(metrics))
}
