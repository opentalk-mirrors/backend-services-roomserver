// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use super::Router;

pub mod rooms;
pub mod signaling;

use opentalk_service_auth::service::ApiKeyAuthorization;
pub use rooms::{RoomAction, RoomBackend};
use signaling::SignalingBackend;

pub trait Backend: Send + Sync + Clone + Sized + RoomBackend + SignalingBackend {}

pub fn routes<B: Backend + 'static>(auth_middleware: ApiKeyAuthorization) -> Router<B> {
    Router::<B>::new()
        .merge(rooms::routes())
        .layer(auth_middleware)
        .merge(signaling::routes())
}
