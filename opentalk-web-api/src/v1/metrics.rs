// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use axum::routing::get;

use crate::{Context, Router};
use axum::extract::State;

pub(crate) async fn metrics(context: State<Context>) -> String {
    context.metric_handle.render()
}

pub(crate) fn routes() -> Router {
    Router::new().route("/metrics", get(metrics))
}
