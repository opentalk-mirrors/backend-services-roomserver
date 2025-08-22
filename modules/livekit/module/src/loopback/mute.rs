// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use futures::{StreamExt as _, stream};
use livekit_api::services::room::RoomClient;
use livekit_protocol::TrackSource;
use opentalk_roomserver_types::connection_id::ConnectionId;
use opentalk_roomserver_types_livekit::ParticipantsMuted;
use opentalk_types_signaling::ParticipantId;
use tracing::{Instrument as _, debug_span};

use crate::{PARALLEL_UPDATES, build_livekit_participant_id};

pub async fn mute_participants(
    livekit_client: Arc<RoomClient>,
    sender: Option<ParticipantId>,
    participants: BTreeMap<ParticipantId, BTreeSet<ConnectionId>>,
    room: String,
) -> ParticipantsMuted {
    let participant_connections = participants
        .into_iter()
        .flat_map(|(p, connections)| {
            let p = std::iter::repeat(p);
            connections.into_iter().zip(p)
        })
        .map(|(c, p)| (p, c, Arc::clone(&livekit_client)));

    let room: &str = &room;
    let muted_participants = stream::iter(participant_connections).map(
        |(participant_id, connection_id, livekit_client)| async move {
            let mute_span = debug_span!("mute", ?participant_id, ?connection_id);
            if let Err(e) = mute(&livekit_client, room, participant_id, connection_id)
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
    participant_id: ParticipantId,
    connection_id: ConnectionId,
) -> anyhow::Result<()> {
    let livekit_participant_id = build_livekit_participant_id(participant_id, connection_id);
    let livekit_participant = livekit_client
        .get_participant(room, &livekit_participant_id)
        .await?;

    for track in livekit_participant.tracks {
        if track.source != TrackSource::Microphone as i32 {
            // Don't mute non-microphone tracks
            tracing::trace!("Skipped muting track, not a microphone.");
            continue;
        }

        livekit_client
            .mute_published_track(room, &livekit_participant_id, &track.sid, true)
            .await?;
        tracing::debug!("Muted participant connection")
    }

    Ok(())
}
