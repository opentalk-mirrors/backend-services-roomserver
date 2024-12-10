// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use axum::extract::ws::Message;
use futures::channel::mpsc;
use opentalk_roomserver_web_api::v1::signaling::websocket;
use opentalk_types_signaling::ParticipantId;

use super::mock_socket::MockSocket;

#[derive(Debug)]
pub struct MockParticipant {
    pub sender: mpsc::Sender<Result<Message, websocket::Error>>,
    #[allow(dead_code)]
    pub receiver: mpsc::Receiver<Message>,
    pub id: ParticipantId,
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
