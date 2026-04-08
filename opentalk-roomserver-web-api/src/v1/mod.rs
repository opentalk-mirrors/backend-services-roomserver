// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use super::Router;
pub use crate::v1::rooms::{RoomAction, RoomBackend};
use crate::v1::{
    livekit_proxy::LiveKitProxyBackend, signaling::SignalingBackend, user::UserBackend,
};

pub mod livekit_proxy;
pub mod rooms;
pub mod signaling;
pub mod user;

use opentalk_service_auth::service::ApiKeyAuthorization;

pub trait Backend:
    Send + Sync + Clone + Sized + RoomBackend + UserBackend + SignalingBackend + LiveKitProxyBackend
{
}

pub fn routes<B: Backend + 'static>(state: B, auth_middleware: ApiKeyAuthorization) -> Router<B> {
    Router::<B>::new()
        .merge(rooms::routes())
        .merge(user::routes())
        .layer(auth_middleware)
        .merge(signaling::routes(state))
        .merge(livekit_proxy::routes())
}
