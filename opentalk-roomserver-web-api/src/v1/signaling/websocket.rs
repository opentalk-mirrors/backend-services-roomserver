// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use axum::extract::ws::WebSocket;
pub use axum::{
    extract::ws::{CloseCode, CloseFrame, Message},
    Error,
};
use futures::{
    stream::{Peekable, SplitSink, SplitStream},
    Sink, Stream,
};

/// A stream of messages for a single signaling connection
pub trait SignalingStream: Stream<Item = Result<Message, axum::Error>> + Send + Unpin {}
impl SignalingStream for WebSocket {}
impl<S: SignalingStream> SignalingStream for Peekable<S> {}
impl<S: SignalingSocket> SignalingStream for SplitStream<S> {}

/// A sink for outgoing messages of a single signaling connection
pub trait SignalingSink: Sink<Message, Error = axum::Error> + Send + Unpin {}
impl SignalingSink for WebSocket {}
impl<S: SignalingSocket> SignalingSink for SplitSink<S, Message> {}

/// A socket implementing both [`SignalingSink`] and [`SignalingStream`].
pub trait SignalingSocket: SignalingStream + SignalingSink {}
impl<S: SignalingSink + SignalingStream> SignalingSocket for S {}
