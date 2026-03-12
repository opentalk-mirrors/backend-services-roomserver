// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use livekit::RoomOptions;
use opentalk_roomserver_module_livekit::LiveKitModule;
use opentalk_roomserver_room::mocking::room::flush_connected_events;
use opentalk_roomserver_types::{
    breakout::breakout_config::{BreakoutConfig, BreakoutRoomConfig},
    room_kind::RoomKind,
};
use opentalk_roomserver_types_livekit::{LiveKitCommand, LiveKitEvent};

mod common;

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_request_access_token() {
    let (_container, mut room, _public_url) = common::build_room().await;

    let mut alice = room.join_alice_moderator(0).await;

    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<LiveKitModule>(LiveKitCommand::RequestPopoutStreamAccessToken, None)
        .await
        .unwrap();
    let token_event = alice.receive_event::<LiveKitModule>().await.unwrap();
    assert!(bob.received_nothing());

    assert!(matches!(
        token_event.payload,
        LiveKitEvent::PopoutStreamAccessToken { .. }
    ));
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_alice_in_breakout_bob_in_main() {
    let (_container, mut room, public_url) = common::build_room().await;

    // Alice and Bob join the meeting
    let mut alice = room.join_alice_moderator(0).await;

    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

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

    alice
        .switch_breakout_room(&mut [&mut bob], RoomKind::Breakout(0.into()))
        .await;

    alice
        .send_command::<LiveKitModule>(LiveKitCommand::RequestPopoutStreamAccessToken, None)
        .await
        .unwrap();
    let token_event = alice.receive_event::<LiveKitModule>().await.unwrap();
    assert!(bob.received_nothing());

    if let LiveKitEvent::PopoutStreamAccessToken { token } = token_event.payload {
        // join livekit to ensure we got the token for the correct room
        let (alice_room, _room_events) =
            livekit::Room::connect(&public_url, &token, RoomOptions::default())
                .await
                .unwrap();
        assert_eq!(alice_room.name(), format!("{}:0", room.id()));
    } else {
        panic!("Expected PopoutStreamAccessToken event, got: {token_event:?}");
    }
}
