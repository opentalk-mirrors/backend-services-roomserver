// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use bytes::Bytes;
use futures::{
    Sink, Stream,
    channel::oneshot,
    stream::{Peekable, SplitSink, SplitStream},
};

#[derive(Debug, thiserror::Error)]
#[error("Websocket error")]
pub struct Error {
    #[from]
    source: Box<dyn std::error::Error + Send + Sync>,
}

impl From<axum::Error> for Error {
    fn from(value: axum::Error) -> Self {
        Self {
            source: Box::new(value),
        }
    }
}

impl From<tokio_util::sync::PollSendError<SignalingSocketMessage>> for Error {
    fn from(value: tokio_util::sync::PollSendError<SignalingSocketMessage>) -> Self {
        Self {
            source: Box::new(value),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CloseFrame {
    pub code: u16,
    pub reason: String,
}

impl From<CloseFrame> for axum::extract::ws::CloseFrame {
    fn from(value: CloseFrame) -> Self {
        Self {
            code: value.code,
            reason: value.reason.into(),
        }
    }
}

impl From<axum::extract::ws::CloseFrame> for CloseFrame {
    fn from(value: axum::extract::ws::CloseFrame) -> Self {
        Self {
            code: value.code,
            reason: value.reason.to_string(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SignalingSocketMessage {
    Text(String),
    Binary(Bytes),
    Ping(Bytes),
    Pong(Bytes),
    Close(Option<CloseFrame>),
}

impl From<String> for SignalingSocketMessage {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<SignalingSocketMessage> for axum::extract::ws::Message {
    fn from(value: SignalingSocketMessage) -> Self {
        match value {
            SignalingSocketMessage::Text(text) => axum::extract::ws::Message::Text(text.into()),
            SignalingSocketMessage::Binary(bytes) => axum::extract::ws::Message::Binary(bytes),
            SignalingSocketMessage::Ping(bytes) => axum::extract::ws::Message::Ping(bytes),
            SignalingSocketMessage::Pong(bytes) => axum::extract::ws::Message::Pong(bytes),
            SignalingSocketMessage::Close(close_frame) => {
                axum::extract::ws::Message::Close(close_frame.map(Into::into))
            }
        }
    }
}

impl From<axum::extract::ws::Message> for SignalingSocketMessage {
    fn from(value: axum::extract::ws::Message) -> Self {
        match value {
            axum::extract::ws::Message::Text(text) => {
                SignalingSocketMessage::Text(text.to_string())
            }
            axum::extract::ws::Message::Binary(bytes) => SignalingSocketMessage::Binary(bytes),
            axum::extract::ws::Message::Ping(bytes) => SignalingSocketMessage::Ping(bytes),
            axum::extract::ws::Message::Pong(bytes) => SignalingSocketMessage::Pong(bytes),
            axum::extract::ws::Message::Close(close_frame) => {
                SignalingSocketMessage::Close(close_frame.map(Into::into))
            }
        }
    }
}

#[derive(Debug)]
pub struct SignalingSocketItem {
    pub message: SignalingSocketMessage,
    pub done: Option<oneshot::Sender<()>>,
}

/// A stream of messages for a single signaling connection
pub trait SignalingStream:
    Stream<Item = Result<SignalingSocketItem, Error>> + Send + Unpin
{
}
impl<S: SignalingStream> SignalingStream for Peekable<S> {}
impl<S: SignalingSocket> SignalingStream for SplitStream<S> {}

/// A sink for outgoing messages of a single signaling connection
pub trait SignalingSink: Sink<SignalingSocketMessage, Error = Error> + Send + Unpin {}
impl<S: SignalingSocket> SignalingSink for SplitSink<S, SignalingSocketMessage> {}

/// A socket implementing both [`SignalingSink`] and [`SignalingStream`].
pub trait SignalingSocket: SignalingStream + SignalingSink {}
impl<S: SignalingSink + SignalingStream> SignalingSocket for S {}
