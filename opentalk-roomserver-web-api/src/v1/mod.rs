// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use super::Router;

pub mod metrics;
pub mod rooms;

pub use metrics::MetricBackend;
pub use rooms::{RoomAction, RoomBackend};

pub trait Backend: RoomBackend + MetricBackend + Send + Sync + Clone + Sized {}

pub fn routes<B: Backend + 'static>() -> Router<B> {
    Router::<B>::new()
        .merge(metrics::routes())
        .merge(rooms::routes())
}
