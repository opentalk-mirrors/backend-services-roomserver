// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use super::Router;

mod middleware;
pub mod rooms;
pub mod signaling;

use middleware::api_token::ServiceAuthLayer;
pub use rooms::{RoomAction, RoomBackend};
use signaling::SignalingBackend;

pub trait Backend: Send + Sync + Clone + Sized + RoomBackend + SignalingBackend {}

pub fn routes<B: Backend + 'static>(api_token: String) -> Router<B> {
    Router::<B>::new()
        .merge(rooms::routes())
        .layer(ServiceAuthLayer { api_token })
        .merge(signaling::routes())
}
