// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
//! Exposed message types
use opentalk_roomserver_types::signaling::SignalingCommand;
use opentalk_types_signaling::ParticipantId;

/// The reason for the closed participant connection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseReason {
    /// The participant initiated the close by sending a WebSocket close frame.
    ParticipantClosed,

    /// The connection to the participant was closed without a close frame.
    ConnectionLost,

    /// The connection to the participant was closed by sending a close frame to the participant.
    TaskClosed,
}

/// A signaling message sent by a client
#[derive(Debug, PartialEq, Eq)]
pub enum SignalingMessage {
    /// The connection to the client was closed
    Closed(CloseReason),
    /// A signaling command for a specific signaling module.
    Command(SignalingCommand),
}

impl SignalingMessage {
    pub fn into_envelope(self, participant_id: ParticipantId) -> MessageEnvelope<SignalingMessage> {
        MessageEnvelope {
            participant_id,
            message: self,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MessageEnvelope<RawMessage> {
    pub participant_id: ParticipantId,
    pub message: RawMessage,
}
