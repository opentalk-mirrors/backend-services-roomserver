// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::BTreeSet;

use livekit::{RoomEvent, RoomOptions};
use livekit_api::services::room::RoomClient;
use opentalk_roomserver_mocking_livekit::{self as mocking, LIVEKIT_KEY, LIVEKIT_SECRET};
use opentalk_roomserver_room::mocking::room::flush_connected_events;
use opentalk_roomserver_types::{
    breakout::{
        breakout_config::{BreakoutConfig, BreakoutRoomConfig},
        event::BreakoutEvent,
    },
    room_kind::RoomKind,
};
use opentalk_roomserver_types_livekit::LiveKitState;

/// Test that the JoinSuccess contains the access token for the LiveKit room.
#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that this test is skipped in the CI. The Livekit server is not
// available there.
async fn livekit_rooms_lifecycle() {
    let (_container, room, public_url) = mocking::build_livekit_room().await;
    let mut room = room.spawn();
    let livekit_client = RoomClient::with_api_key(&public_url, LIVEKIT_KEY, LIVEKIT_SECRET);

    let mut alice = room.join_alice_moderator(0).await;

    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let bob_livekit_state = bob
        .join_success()
        .get_module::<LiveKitState>()
        .expect("LiveKit state must be deserializable")
        .expect("LiveKit state must be present");

    // join livekit to ensure participant exists and can be muted.
    let (_bob_room, mut room_events) = livekit::Room::connect(
        &public_url,
        &bob_livekit_state.credentials.token,
        RoomOptions::default(),
    )
    .await
    .unwrap();
    let connected = room_events.recv().await;
    assert!(matches!(connected, Some(RoomEvent::Connected { .. })));

    // start breakout rooms
    alice
        .start_breakout_rooms(
            &mut [&mut bob],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "Room 0".to_string(),
                    assignments: vec![],
                }],
                duration: None,
            },
        )
        .await;

    let event = alice
        .switch_breakout_room(&mut [&mut bob], RoomKind::Breakout(0.into()))
        .await;

    let BreakoutEvent::SwitchedRoom { own_data, .. } = event else {
        panic!("Expected SwitchedRoom, got: {event:?}")
    };

    let alice_livekit_state = own_data
        .get::<LiveKitState>()
        .expect("LiveKit state must be deserializable")
        .expect("LiveKit state must be present");

    // join livekit to ensure participant exists and can be muted.
    let (_bob_room, mut room_events) = livekit::Room::connect(
        &public_url,
        &alice_livekit_state.credentials.token,
        RoomOptions::default(),
    )
    .await
    .unwrap();
    let connected = room_events.recv().await;
    assert!(matches!(connected, Some(RoomEvent::Connected { .. })));

    // livekit rooms should be created
    let room_list: BTreeSet<_> = livekit_client
        .list_rooms(vec![])
        .await
        .unwrap()
        .into_iter()
        .map(|room| room.name)
        .collect();
    let room_id = room.id();
    assert_eq!(
        room_list,
        BTreeSet::from([format!("{room_id}:main"), format!("{room_id}:0")])
    );

    // Stop breakout rooms, livekit rooms should be cleaned up
    alice
        .switch_breakout_room(&mut [&mut bob], RoomKind::Main)
        .await;
    alice.stop_breakout_rooms(&mut [&mut bob]).await;

    let room_list: BTreeSet<_> = livekit_client
        .list_rooms(vec![])
        .await
        .unwrap()
        .into_iter()
        .map(|room| room.name)
        .collect();
    let room_id = room.id();
    assert_eq!(room_list, BTreeSet::from([format!("{room_id}:main")]));

    alice.disconnect().await.unwrap();
    bob.disconnect().await.unwrap();

    // there is no way to wait for the room to be destroyed
}
