// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::BTreeSet, sync::Arc};

use livekit_api::services::room::RoomClient;
use opentalk_roomserver_types::connection_id::ConnectionId;
use opentalk_roomserver_types_livekit::error::LiveKitError;
use opentalk_types_signaling::ParticipantId;

use crate::loopback::LiveKitLoopback;

pub async fn revoke_token(
    livekit_client: Arc<RoomClient>,
    participant_id: ParticipantId,
    connection_id: ConnectionId,
    room: String,
    token_identities: BTreeSet<String>,
) -> Result<LiveKitLoopback, LiveKitError> {
    let mut revoked_identities = BTreeSet::new();
    for identity in token_identities {
        if let Err(e) = livekit_client.remove_participant(&room, &identity).await {
            tracing::error!("Failed to revoke token {e}");
        } else {
            revoked_identities.insert(identity);
        }
    }
    Ok(LiveKitLoopback::NoteRevokedTokens {
        participant_id,
        connection_id,
        token_identities: revoked_identities,
    })
}
