// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use axum::extract::ws::Message;
use futures::{
    SinkExt as _, StreamExt,
    channel::mpsc::{self, SendError},
};
use opentalk_roomserver_web_api::v1::signaling::websocket;
use opentalk_types_signaling::ParticipantId;
use serde_json::json;

use super::mock_socket::MockSocket;

#[derive(Debug)]
pub struct MockParticipant {
    pub sender: mpsc::Sender<Result<Message, websocket::Error>>,
    #[allow(dead_code)]
    pub receiver: mpsc::Receiver<Message>,
    pub id: ParticipantId,
}

impl MockParticipant {
    async fn static_send_ping(
        sender: &mut mpsc::Sender<Result<Message, websocket::Error>>,
    ) -> Result<(), SendError> {
        sender
            .send(Ok(axum::extract::ws::Message::Text(
                json!( {
                    "namespace": "ping",
                    "content": serde_json::Value::Null,
                })
                .to_string()
                .into(),
            )))
            .await
    }
    #[allow(dead_code)]
    pub async fn send_ping(&mut self) -> Result<(), SendError> {
        Self::static_send_ping(&mut self.sender).await
    }

    pub fn queue_send_ping(&self) {
        let mut sender = self.sender.clone();
        tokio::spawn(async move {
            Self::static_send_ping(&mut sender)
                .await
                .expect("Send should succeed eventually");
        });
    }

    #[allow(dead_code)]
    pub async fn receive_event(&mut self) -> Option<Message> {
        self.receiver.next().await
    }
}

pub fn create_participant_connection() -> (MockSocket, MockParticipant) {
    let websocket_in = mpsc::channel(1);
    let websocket_out = mpsc::channel(1);
    (
        MockSocket::new(websocket_in.1, websocket_out.0),
        MockParticipant {
            sender: websocket_in.0,
            receiver: websocket_out.1,
            id: ParticipantId::generate(),
        },
    )
}
