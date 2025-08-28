// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use futures::{StreamExt as _, stream};
use livekit_api::services::room::RoomClient;
use livekit_protocol::TrackSource;
use opentalk_roomserver_types_livekit::ParticipantsMuted;
use opentalk_types_signaling::ParticipantId;
use tracing::{Instrument as _, debug_span};

use crate::{LiveKitConnection, PARALLEL_UPDATES};

pub async fn mute_participants(
    sender: Option<ParticipantId>,
    participant_connections: Vec<LiveKitConnection>,
) -> ParticipantsMuted {
    let muted_participants = stream::iter(participant_connections).map(
        |LiveKitConnection { participant_id, livekit_participant_id, livekit_room: room, livekit_client }| async move {
            let mute_span = debug_span!("mute", livekit_participant_id);
            if let Err(e) = mute(&livekit_client, &room, &livekit_participant_id)
                .instrument(mute_span.clone())
                .await
            {
                // The participant might not have a microphone or already left the meeting.
                tracing::debug!(parent: &mute_span, "failed to mute participant connection: {e}");
            }
            participant_id
        },
    ).buffer_unordered(PARALLEL_UPDATES).collect().await;

    ParticipantsMuted {
        sender,
        participants: muted_participants,
    }
}

async fn mute(
    livekit_client: &Arc<RoomClient>,
    room: &str,
    livekit_participant_id: &str,
) -> anyhow::Result<()> {
    let livekit_participant = livekit_client
        .get_participant(room, livekit_participant_id)
        .await?;

    for track in livekit_participant.tracks {
        if track.source != TrackSource::Microphone as i32 {
            // Don't mute non-microphone tracks
            tracing::trace!("Skipped muting track, not a microphone.");
            continue;
        }

        livekit_client
            .mute_published_track(room, livekit_participant_id, &track.sid, true)
            .await?;
        tracing::debug!("Muted participant connection")
    }

    Ok(())
}
