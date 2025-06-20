// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use livekit_api::services::room::{CreateRoomOptions, RoomClient};
use opentalk_roomserver_types_livekit::error::LiveKitError;
use opentalk_types_common::rooms::RoomId;

use crate::loopback::LiveKitLoopback;

pub async fn create_room(
    livekit_client: Arc<RoomClient>,
    room_id: RoomId,
) -> Result<LiveKitLoopback, LiveKitError> {
    let room = livekit_client
        .create_room(&room_id.to_string(), CreateRoomOptions::default())
        .await
        .map_err(|err| {
            tracing::error!("failed to create room: {}", err);
            LiveKitError::LivekitUnavailable
        })?;
    tracing::debug!("LiveKit room created: {} (room-id: {})", room.name, room_id);
    Ok(LiveKitLoopback::RoomCreated)
}
