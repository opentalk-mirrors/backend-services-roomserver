// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use axum::extract::ws::WebSocket;
pub use axum::{
    extract::ws::{CloseCode, CloseFrame, Message},
    Error,
};
use futures::{Sink, Stream};

pub trait SignalingSocket:
    Stream<Item = Result<Message, axum::Error>> + Sink<Message, Error = axum::Error> + Send + Unpin
{
}

impl SignalingSocket for WebSocket {}
