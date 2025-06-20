// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::BTreeSet;

use livekit::RoomOptions;
use opentalk_roomserver_module_livekit::LiveKitModule;
use opentalk_roomserver_types::core_event::CoreEvent;
use opentalk_roomserver_types_livekit::{
    command::LiveKitCommand, error::LiveKitError, event::LiveKitEvent,
};
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_livekit::state::LiveKitState;
use pretty_assertions::assert_eq;

mod common;

#[test_log::test(tokio::test)]
#[ignore]
async fn unknown_participant() {
    let disconnected_participant = ParticipantId::from_u128(0x461ba262_6bb1_4c85_bbd5_b3d010b1a076);
    let (_container, mut room, _public_url) = common::build_livekit_room().await;

    // Alice joins the meeting
    let mut alice = room.join_alice_moderator().await;

    // Alice sends the command
    alice
        .send_command::<LiveKitModule>(
            LiveKitCommand::RevokeScreenSharePermission {
                participants: BTreeSet::from_iter(vec![disconnected_participant]),
            },
            None,
        )
        .await
        .unwrap();

    let error_event = alice.receive_event::<LiveKitModule>().await.unwrap();
    assert_eq!(
        error_event.content,
        LiveKitEvent::Error(LiveKitError::UnknownParticipant {
            participant: BTreeSet::from_iter(vec![disconnected_participant])
        })
    )
}

#[test_log::test(tokio::test)]
#[ignore]
async fn insufficient_permissions() {
    let disconnected_participant = ParticipantId::from_u128(0x461ba262_6bb1_4c85_bbd5_b3d010b1a076);
    let (_container, mut room, _public_url) = common::build_livekit_room().await;

    // Bob joins the meeting
    let mut bob = room.join_bob().await;

    // Bob sends the command
    bob.send_command::<LiveKitModule>(
        LiveKitCommand::RevokeScreenSharePermission {
            participants: BTreeSet::from_iter(vec![disconnected_participant]),
        },
        None,
    )
    .await
    .unwrap();

    let error_event = bob.receive_event::<LiveKitModule>().await.unwrap();
    assert_eq!(
        error_event.content,
        LiveKitEvent::Error(LiveKitError::InsufficientPermissions)
    )
}

#[test_log::test(tokio::test)]
#[ignore]
async fn revoke_bob() {
    let (_container, mut room, public_url) = common::build_livekit_room().await;

    // Alice and Bob join the meeting
    let mut alice = room.join_alice_moderator().await;

    let mut bob = room.join_bob().await;
    let bob_livekit_state = bob
        .join_success()
        .get_module::<LiveKitState>()
        .expect("LiveKit state must be deserializable")
        .expect("LiveKit state must be present");

    assert!(matches!(
        alice.receive::<CoreEvent>().await.unwrap().content,
        CoreEvent::ParticipantConnected { .. }
    ));
    // join livekit to ensure participant exists
    let (_room, _room_events) = livekit::Room::connect(
        &public_url,
        &bob_livekit_state.credentials.token,
        RoomOptions::default(),
    )
    .await
    .unwrap();

    // Alice sends the command
    alice
        .send_command::<LiveKitModule>(
            LiveKitCommand::RevokeScreenSharePermission {
                participants: BTreeSet::from_iter(vec![bob.id()]),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice.receive_event::<LiveKitModule>().await.unwrap();
    assert_eq!(
        event.content,
        LiveKitEvent::ScreenSharePermissionsUpdated {
            grant: false,
            participants: vec![bob.id()],
        }
    );

    // Bob is notified by the livekit signaling session (separate to our signaling)
    assert!(bob.received_nothing());
}
