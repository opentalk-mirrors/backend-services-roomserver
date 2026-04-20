// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

#[cfg(feature = "axum")]
pub mod adapter;
pub mod websocket;

use opentalk_types_common::rooms::RoomId;
use opentalk_types_signaling::ParticipantId;
use tokio_tungstenite::MaybeTlsStream;
use uuid::Uuid;

use crate::{connection_id::ConnectionId, room_kind::RoomKind};

#[derive(Debug, Clone)]
pub enum LiveKitAccessToken {
    Header(String),
    Query(String),
}

impl LiveKitAccessToken {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Header(token) | Self::Query(token) => token,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LiveKitProxyTarget {
    LiveKit { room_kind: RoomKind },
    SubroomAudio { whisper_id: Uuid },
}

#[derive(Debug, Clone)]
pub struct LiveKitProxyRequest {
    pub access_token: LiveKitAccessToken,
    pub room_id: RoomId,
    pub proxy_target: LiveKitProxyTarget,
    pub participant_id: ParticipantId,
    pub connection_id: ConnectionId,
}

/// The type of an upstream WebSocket connection to a LiveKit server.
pub type PreparedSocket = tokio_tungstenite::WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
