// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_mocking_livekit as mocking;
use opentalk_roomserver_types_livekit::{LiveKitState, MicrophoneRestrictionState};
use pretty_assertions::assert_eq;

/// Test that the JoinSuccess contains the access token for the LiveKit room.
#[test_log::test(tokio::test)]
#[ignore]
async fn joined_participant_receives_key() {
    let (_container, room, livekit_url) = mocking::build_livekit_room().await;
    let mut room = room.spawn();

    let alice = room.join_alice_moderator(0).await;
    let alice_livekit_state = alice
        .join_success()
        .get_module::<LiveKitState>()
        .expect("LiveKit state must be deserializable")
        .expect("LiveKit state must be present");

    assert_eq!(
        alice_livekit_state.microphone_restriction_state,
        MicrophoneRestrictionState::Disabled,
    );
    assert_eq!(
        alice_livekit_state.credentials.room,
        format!("{}:main", room.id())
    );
    assert!(!alice_livekit_state.credentials.token.is_empty());
    assert!(alice_livekit_state.credentials.service_url.is_none());
    assert_eq!(alice_livekit_state.credentials.public_url, livekit_url);
}
