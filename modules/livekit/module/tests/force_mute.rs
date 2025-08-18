// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::BTreeSet;

use livekit::{RoomEvent, RoomOptions};
use opentalk_roomserver_module_livekit::LiveKitModule;
use opentalk_roomserver_room::mocking::room::flush_connected_events;
use opentalk_roomserver_types::{
    breakout::breakout_config::{BreakoutConfig, BreakoutRoomConfig},
    room_kind::RoomKind,
};
use opentalk_roomserver_types_livekit::{LiveKitCommand, LiveKitError, LiveKitEvent, LiveKitState};
use opentalk_types_signaling::ParticipantId;
use pretty_assertions::assert_eq;

mod common;

#[test_log::test(tokio::test)]
#[ignore]
async fn unknown_participant() {
    let (_container, mut room, _public_url) = common::build_livekit_room().await;

    let disconnected_participant = ParticipantId::from_u128(0x461ba262_6bb1_4c85_bbd5_b3d010b1a076);

    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<LiveKitModule>(
            LiveKitCommand::ForceMute {
                participants: BTreeSet::from([disconnected_participant]),
            },
            None,
        )
        .await
        .unwrap();

    let error_event = alice.receive_event::<LiveKitModule>().await.unwrap();
    assert_eq!(
        error_event.payload,
        LiveKitEvent::Error(LiveKitError::UnknownParticipant {
            participant: BTreeSet::from([disconnected_participant])
        })
    )
}

#[test_log::test(tokio::test)]
#[ignore]
async fn mute_bob() {
    let (_container, mut room, public_url) = common::build_livekit_room().await;

    // Alice and Bob join the meeting
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
    // log additional livekit events
    tokio::spawn(async move {
        while let Some(event) = room_events.recv().await {
            tracing::debug!("Bob Livekit Event: {:?}", event);
        }
        tracing::debug!("Bob Livekit Event stream closed");
    });

    // Alice sends the force mute command
    alice
        .send_command::<LiveKitModule>(
            LiveKitCommand::ForceMute {
                participants: BTreeSet::from([bob.id()]),
            },
            None,
        )
        .await
        .unwrap();

    assert!(alice.received_nothing());

    let force_mute_event = bob.receive_event::<LiveKitModule>().await.unwrap();
    assert_eq!(
        force_mute_event.payload,
        LiveKitEvent::ForceMuted {
            moderator: alice.id()
        }
    );
}

#[test_log::test(tokio::test)]
#[ignore]
async fn insufficient_permissions() {
    let disconnected_participant = ParticipantId::from_u128(0x461ba262_6bb1_4c85_bbd5_b3d010b1a076);
    let (_container, mut room, _public_url) = common::build_livekit_room().await;

    // Bob joins the meeting
    let mut bob = room.join_bob(0).await;

    // Bob sends the command
    bob.send_command::<LiveKitModule>(
        LiveKitCommand::ForceMute {
            participants: BTreeSet::from([disconnected_participant]),
        },
        None,
    )
    .await
    .unwrap();

    let error_event = bob.receive_event::<LiveKitModule>().await.unwrap();
    assert_eq!(
        error_event.payload,
        LiveKitEvent::Error(LiveKitError::InsufficientPermissions)
    )
}

#[test_log::test(tokio::test)]
#[ignore]
async fn alice_in_breakout_bob_in_main() {
    let (_container, mut room, _public_url) = common::build_livekit_room().await;

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

    // Alice sends the force mute command
    alice
        .send_command::<LiveKitModule>(
            LiveKitCommand::ForceMute {
                participants: BTreeSet::from([bob.id()]),
            },
            None,
        )
        .await
        .unwrap();

    assert!(alice.received_nothing());

    let force_mute_event = bob.receive_event::<LiveKitModule>().await.unwrap();
    assert_eq!(
        force_mute_event.payload,
        LiveKitEvent::ForceMuted {
            moderator: alice.id()
        }
    );
}
