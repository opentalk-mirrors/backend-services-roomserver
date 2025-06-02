// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_signaling_livekit::{MicrophoneRestrictionState, state::LiveKitState};
use pretty_assertions::assert_eq;

mod common;

/// Test that the JoinSuccess contains the access token for the LiveKit room.
#[test_log::test(tokio::test)]
#[ignore]
async fn joined_participant_receives_key() {
    let (_container, mut room, livekit_url) = common::build_livekit_room().await;

    let alice = room.join_alice_moderator().await;
    let alice_livekit_state = alice
        .join_success()
        .get_module::<LiveKitState>()
        .expect("LiveKit state must be deserializable")
        .expect("LiveKit state must be present");

    assert_eq!(
        alice_livekit_state.microphone_restriction_state,
        MicrophoneRestrictionState::Disabled,
    );
    assert_eq!(alice_livekit_state.credentials.room, room.id().to_string());
    assert!(!alice_livekit_state.credentials.token.is_empty());
    assert!(alice_livekit_state.credentials.service_url.is_none());
    assert_eq!(alice_livekit_state.credentials.public_url, livekit_url);
}
