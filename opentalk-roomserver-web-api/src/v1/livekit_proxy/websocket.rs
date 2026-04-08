// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use axum::extract::ws::Message;
use bytes::Bytes;
use futures::{
    Sink, Stream,
    stream::{Peekable, SplitSink, SplitStream},
};
use tokio_util::sync::PollSendError;

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

impl From<PollSendError<LiveKitSocketMessage>> for Error {
    fn from(value: PollSendError<LiveKitSocketMessage>) -> Self {
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
pub enum LiveKitSocketMessage {
    Text(String),
    Binary(Bytes),
    Ping(Bytes),
    Pong(Bytes),
    Close(Option<CloseFrame>),
}

impl From<String> for LiveKitSocketMessage {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<LiveKitSocketMessage> for Message {
    fn from(value: LiveKitSocketMessage) -> Self {
        match value {
            LiveKitSocketMessage::Text(text) => Message::Text(text.into()),
            LiveKitSocketMessage::Binary(bytes) => Message::Binary(bytes),
            LiveKitSocketMessage::Ping(bytes) => Message::Ping(bytes),
            LiveKitSocketMessage::Pong(bytes) => Message::Pong(bytes),
            LiveKitSocketMessage::Close(close_frame) => Message::Close(close_frame.map(Into::into)),
        }
    }
}

impl From<Message> for LiveKitSocketMessage {
    fn from(value: Message) -> Self {
        match value {
            Message::Text(text) => LiveKitSocketMessage::Text(text.to_string()),
            Message::Binary(bytes) => LiveKitSocketMessage::Binary(bytes),
            Message::Ping(bytes) => LiveKitSocketMessage::Ping(bytes),
            Message::Pong(bytes) => LiveKitSocketMessage::Pong(bytes),
            Message::Close(close_frame) => LiveKitSocketMessage::Close(close_frame.map(Into::into)),
        }
    }
}

/// A stream of messages for a single signaling connection
pub trait LiveKitStream: Stream<Item = Result<LiveKitSocketMessage, Error>> + Send + Unpin {}
impl<S: LiveKitStream> LiveKitStream for Peekable<S> {}
impl<S: LiveKitSocket> LiveKitStream for SplitStream<S> {}

/// A sink for outgoing messages of a single signaling connection
pub trait LiveKitSink: Sink<LiveKitSocketMessage, Error = Error> + Send + Unpin {}
impl<S: LiveKitSocket> LiveKitSink for SplitSink<S, LiveKitSocketMessage> {}

/// A socket implementing both [`LiveKitSink`] and [`LiveKitStream`].
pub trait LiveKitSocket: LiveKitStream + LiveKitSink {}
impl<S: LiveKitSink + LiveKitStream> LiveKitSocket for S {}
