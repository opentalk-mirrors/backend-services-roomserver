// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use axum::{async_trait, routing::get};

use crate::Router;
use axum::extract::State;

#[async_trait]
pub trait MetricHandle: Clone + Send + Sync {
    async fn render(&mut self) -> String;
}

pub(crate) async fn metrics<Api: MetricHandle>(mut context: State<Api>) -> String {
    context.render().await
}

pub(crate) fn routes<Api: MetricHandle + 'static>() -> Router<Api> {
    Router::<Api>::new().route("/metrics", get(metrics::<Api>))
}
