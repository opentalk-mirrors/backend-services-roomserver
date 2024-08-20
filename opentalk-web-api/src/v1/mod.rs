// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use super::Router;

mod metrics;
mod rooms;

pub(crate) fn routes() -> Router {
    Router::new()
        .merge(metrics::routes())
        .merge(rooms::routes())
}
