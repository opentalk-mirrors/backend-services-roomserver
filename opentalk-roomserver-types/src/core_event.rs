// SPDX-License-Identifier: EUPL-1.2
//
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::BTreeMap;

use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::{
    connection_id::ConnectionId, disconnect_reason::DisconnectReason,
    join::join_success::JoinSuccess, shared_raw_json::SharedRawJson,
};

/// Outgoing websocket messages in the core namespace
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoreEvent {
    /// Message sent to a participant on a successful join
    JoinSuccess(Box<JoinSuccess>),

    /// Broadcast message sent to all participants when a new participant has joined
    ParticipantConnected {
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        peer_join_info: BTreeMap<ModuleId, SharedRawJson>, // TODO: find a better name
    },

    /// Broadcast message sent to all participants when a participant disconnected
    ParticipantDisconnected {
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        reason: DisconnectReason,
    },
}
