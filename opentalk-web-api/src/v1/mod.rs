// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use super::Router;

mod metrics;
mod rooms;

pub use metrics::MetricHandle;
pub use rooms::RoomContext;

pub trait RoomServerApi: RoomContext + MetricHandle + Send + Sync + Clone + Sized {}

pub fn routes<Api: RoomServerApi + 'static>() -> Router<Api> {
    Router::<Api>::new()
        .merge(metrics::routes())
        .merge(rooms::routes())
}
