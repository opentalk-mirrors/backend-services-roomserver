// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use axum::{async_trait, routing::get};

use crate::Router;
use axum::extract::State;

#[async_trait]
pub trait MetricBackend: Clone + Send + Sync {
    async fn render(&mut self) -> String;
}

pub(crate) async fn metrics<B: MetricBackend>(mut context: State<B>) -> String {
    context.render().await
}

pub(crate) fn routes<B: MetricBackend + 'static>() -> Router<B> {
    Router::<B>::new().route("/metrics", get(metrics::<B>))
}
