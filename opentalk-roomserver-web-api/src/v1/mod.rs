// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use super::Router;

pub mod rooms;
pub mod signaling;

pub use rooms::{RoomAction, RoomBackend};
use signaling::SignalingBackend;

pub trait Backend: Send + Sync + Clone + Sized + RoomBackend + SignalingBackend {}

pub fn routes<B: Backend + 'static>() -> Router<B> {
    Router::<B>::new()
        .merge(rooms::routes())
        .merge(signaling::routes())
}
