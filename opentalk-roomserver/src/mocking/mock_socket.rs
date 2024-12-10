// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use axum::extract::ws::Message;
use futures::{channel::mpsc, Sink, SinkExt, Stream, TryStreamExt};
use opentalk_roomserver_web_api::v1::signaling::websocket::{self, SignalingSocket};

#[derive(Debug)]
pub struct MockSocket {
    receiver: mpsc::Receiver<Result<Message, websocket::Error>>,
    sender: mpsc::Sender<Message>,
}

impl MockSocket {
    pub fn new(
        receiver: mpsc::Receiver<Result<Message, websocket::Error>>,
        sender: mpsc::Sender<Message>,
    ) -> MockSocket {
        MockSocket { receiver, sender }
    }
}

impl Stream for MockSocket {
    type Item = Result<Message, websocket::Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.receiver.try_poll_next_unpin(cx)
    }
}

impl Sink<Message> for MockSocket {
    type Error = websocket::Error;

    fn poll_ready(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.sender.poll_ready_unpin(cx).map_err(axum::Error::new)
    }

    fn start_send(mut self: std::pin::Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        self.sender.start_send_unpin(item).map_err(axum::Error::new)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.sender.poll_flush_unpin(cx).map_err(axum::Error::new)
    }

    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.sender.poll_close_unpin(cx).map_err(axum::Error::new)
    }
}

impl SignalingSocket for MockSocket {}
