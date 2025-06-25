// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::BTreeSet;

use livekit::{RoomEvent, RoomOptions};
use opentalk_roomserver_module_livekit::LiveKitModule;
use opentalk_roomserver_room::mocking::room::flush_connected_events;
use opentalk_roomserver_types::core_event::CoreEvent;
use opentalk_roomserver_types_livekit::{
    command::LiveKitCommand, error::LiveKitError, event::LiveKitEvent,
};
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_livekit::{command::UnrestrictedParticipants, state::LiveKitState};
use pretty_assertions::assert_eq;

mod common;

#[test_log::test(tokio::test)]
#[ignore]
async fn microphones_are_restricted() {
    let (_container, mut room, public_url) = common::build_livekit_room().await;

    // Alice joins the meeting
    let mut alice = room.join_alice_moderator(0).await;

    // Bob joins the meeting
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let bob_livekit_state = bob
        .join_success()
        .get_module::<LiveKitState>()
        .expect("LiveKit state must be deserializable")
        .expect("LiveKit state must be present");

    // join livekit to ensure participant exists and can be muted.
    let (bob_room, mut room_events) = livekit::Room::connect(
        &public_url,
        &bob_livekit_state.credentials.token,
        RoomOptions::default(),
    )
    .await
    .unwrap();
    let connected = room_events.recv().await;
    assert!(matches!(connected, Some(RoomEvent::Connected { .. })));
    // Bob should be able to publish audio
    common::publish_audio(&bob_room).await.unwrap();

    let unrestricted = BTreeSet::from_iter([alice.id()]);

    // Alice sends the command
    alice
        .send_command::<LiveKitModule>(
            LiveKitCommand::EnableMicrophoneRestrictions(UnrestrictedParticipants {
                unrestricted_participants: unrestricted.clone(),
            }),
            None,
        )
        .await
        .unwrap();

    let success_event = alice.receive_event::<LiveKitModule>().await.unwrap();

    assert_eq!(
        success_event.content,
        LiveKitEvent::MicrophoneRestrictionsEnabled(UnrestrictedParticipants {
            unrestricted_participants: unrestricted.clone(),
        })
    );

    // Bob should receive restriction state update
    let force_mute_event = bob.receive_event::<LiveKitModule>().await.unwrap();
    assert_eq!(
        force_mute_event.content,
        LiveKitEvent::MicrophoneRestrictionsEnabled(UnrestrictedParticipants {
            unrestricted_participants: unrestricted
        })
    );

    // Bob should not be able to send audio
    assert!(common::publish_audio(&bob_room).await.is_err());
}

/// When the restricted state is updated, the permissions are adjusted accordingly.
///
/// e.g. bob was restricted before, but is added to the unrestricted set.
#[test_log::test(tokio::test)]
#[ignore]
async fn permissions_are_updated() {
    let (_container, mut room, public_url) = common::build_livekit_room().await;

    // Alice joins the meeting
    let mut alice = room.join_alice_moderator(0).await;

    // Bob joins the meeting
    let mut bob = room.join_bob(0).await;
    let bob_livekit_state = bob
        .join_success()
        .get_module::<LiveKitState>()
        .expect("LiveKit state must be deserializable")
        .expect("LiveKit state must be present");

    assert!(matches!(
        alice.receive::<CoreEvent>().await.unwrap().content,
        CoreEvent::ParticipantConnected { .. }
    ));
    // join livekit to ensure participant exists and can be muted.
    let (bob_room, mut room_events) = livekit::Room::connect(
        &public_url,
        &bob_livekit_state.credentials.token,
        RoomOptions::default(),
    )
    .await
    .unwrap();
    let connected = room_events.recv().await;
    assert!(matches!(connected, Some(RoomEvent::Connected { .. })));
    // Bob should be able to publish audio
    common::publish_audio(&bob_room).await.unwrap();

    let unrestricted = BTreeSet::from_iter([alice.id()]);

    // Alice sends the command to restrict bob
    alice
        .send_command::<LiveKitModule>(
            LiveKitCommand::EnableMicrophoneRestrictions(UnrestrictedParticipants {
                unrestricted_participants: unrestricted.clone(),
            }),
            None,
        )
        .await
        .unwrap();

    let success_event = alice.receive_event::<LiveKitModule>().await.unwrap();
    assert_eq!(
        success_event.content,
        LiveKitEvent::MicrophoneRestrictionsEnabled(UnrestrictedParticipants {
            unrestricted_participants: unrestricted.clone()
        })
    );

    // Bob should receive restriction state update
    let force_mute_event = bob.receive_event::<LiveKitModule>().await.unwrap();
    assert_eq!(
        force_mute_event.content,
        LiveKitEvent::MicrophoneRestrictionsEnabled(UnrestrictedParticipants {
            unrestricted_participants: unrestricted.clone()
        })
    );

    let unrestricted = BTreeSet::from_iter([alice.id(), bob.id()]);

    // Alice sends the command to lift bobs restrictions
    alice
        .send_command::<LiveKitModule>(
            LiveKitCommand::EnableMicrophoneRestrictions(UnrestrictedParticipants {
                unrestricted_participants: unrestricted.clone(),
            }),
            None,
        )
        .await
        .unwrap();

    let success_event = alice.receive_event::<LiveKitModule>().await.unwrap();
    assert_eq!(
        success_event.content,
        LiveKitEvent::MicrophoneRestrictionsEnabled(UnrestrictedParticipants {
            unrestricted_participants: unrestricted.clone()
        })
    );

    // Bob should receive restriction state update
    let force_mute_event = bob.receive_event::<LiveKitModule>().await.unwrap();
    assert_eq!(
        force_mute_event.content,
        LiveKitEvent::MicrophoneRestrictionsEnabled(UnrestrictedParticipants {
            unrestricted_participants: unrestricted
        })
    );

    common::publish_audio(&bob_room)
        .await
        .expect("Publishing audio must work again");
}

#[test_log::test(tokio::test)]
#[ignore]
async fn enable_unknown_participant() {
    let disconnected_participant = ParticipantId::from_u128(0x461ba262_6bb1_4c85_bbd5_b3d010b1a076);
    let (_container, mut room, _public_url) = common::build_livekit_room().await;

    // Alice joins the meeting
    let mut alice = room.join_alice_moderator(0).await;

    let disconnected = BTreeSet::from_iter([disconnected_participant]);

    // Alice sends the command
    alice
        .send_command::<LiveKitModule>(
            LiveKitCommand::EnableMicrophoneRestrictions(UnrestrictedParticipants {
                unrestricted_participants: disconnected.clone(),
            }),
            None,
        )
        .await
        .unwrap();

    // We allow for unknown participants in the unrestricted set. If participants disconnect their
    // permission is kept.
    let success_event = alice.receive_event::<LiveKitModule>().await.unwrap();
    assert_eq!(
        success_event.content,
        LiveKitEvent::MicrophoneRestrictionsEnabled(UnrestrictedParticipants {
            unrestricted_participants: disconnected
        })
    );
}

/// Alice enables restrictions except for bob, bob leaves the meeting, restrictions are lifted.
/// At the time of lifting the restrictions bob is still in the state but not a participant anymore.
#[test_log::test(tokio::test)]
#[ignore]
async fn disable_unknown_participant() {
    let (_container, mut room, public_url) = common::build_livekit_room().await;

    // Alice joins the meeting
    let mut alice = room.join_alice_moderator(0).await;

    // Bob joins the meeting
    let bob = room.join_bob(0).await;
    let bob_livekit_state = bob
        .join_success()
        .get_module::<LiveKitState>()
        .expect("LiveKit state must be deserializable")
        .expect("LiveKit state must be present");

    assert!(matches!(
        alice.receive::<CoreEvent>().await.unwrap().content,
        CoreEvent::ParticipantConnected { .. }
    ));
    {
        // join livekit to ensure participant exists and can be muted.
        let (bob_room, mut room_events) = livekit::Room::connect(
            &public_url,
            &bob_livekit_state.credentials.token,
            RoomOptions::default(),
        )
        .await
        .unwrap();
        let connected = room_events.recv().await;
        assert!(matches!(connected, Some(RoomEvent::Connected { .. })));
        // Bob should be able to publish audio
        common::publish_audio(&bob_room).await.unwrap();

        let unrestricted = BTreeSet::from_iter([alice.id()]);

        // Alice sends the command
        alice
            .send_command::<LiveKitModule>(
                LiveKitCommand::EnableMicrophoneRestrictions(UnrestrictedParticipants {
                    unrestricted_participants: unrestricted,
                }),
                None,
            )
            .await
            .unwrap();
        let _restrictions_enabled = alice.receive_event::<LiveKitModule>().await.unwrap();
    }
    bob.disconnect();
    let _bob_left = alice.receive::<CoreEvent>().await.unwrap();

    // Alice sends the command
    alice
        .send_command::<LiveKitModule>(LiveKitCommand::DisableMicrophoneRestrictions, None)
        .await
        .unwrap();

    // We allow for unknown participants in the unrestricted set. If participants disconnect their
    // permission is kept.
    let success_event = alice.receive_event::<LiveKitModule>().await.unwrap();
    assert_eq!(
        success_event.content,
        LiveKitEvent::MicrophoneRestrictionsDisabled
    );
}

#[test_log::test(tokio::test)]
#[ignore]
async fn disable_insufficient_permissions() {
    let (_container, mut room, _public_url) = common::build_livekit_room().await;

    // Bob joins the meeting
    let mut bob = room.join_bob(0).await;

    // Bob sends the command
    bob.send_command::<LiveKitModule>(LiveKitCommand::DisableMicrophoneRestrictions, None)
        .await
        .unwrap();

    let error_event = bob.receive_event::<LiveKitModule>().await.unwrap();
    assert_eq!(
        error_event.content,
        LiveKitEvent::Error(LiveKitError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
#[ignore]
async fn enable_insufficient_permissions() {
    let disconnected_participant = ParticipantId::from_u128(0x461ba262_6bb1_4c85_bbd5_b3d010b1a076);
    let (_container, mut room, _public_url) = common::build_livekit_room().await;

    // Bob joins the meeting
    let mut bob = room.join_bob(0).await;

    let unrestricted = BTreeSet::from_iter([disconnected_participant]);

    // Bob sends the command
    bob.send_command::<LiveKitModule>(
        LiveKitCommand::EnableMicrophoneRestrictions(UnrestrictedParticipants {
            unrestricted_participants: unrestricted,
        }),
        None,
    )
    .await
    .unwrap();

    let error_event = bob.receive_event::<LiveKitModule>().await.unwrap();
    assert_eq!(
        error_event.content,
        LiveKitEvent::Error(LiveKitError::InsufficientPermissions)
    );
}

/// The [`LiveKitModule::ongoing_microphone_restrictions`] barrier should be freed after the operation finished.
#[test_log::test(tokio::test)]
#[ignore]
async fn barrier_should_be_freed() {
    let (_container, mut room, _public_url) = common::build_livekit_room().await;

    // Alice joins the meeting
    let mut alice = room.join_alice_moderator(0).await;

    // Alice sends the command
    for _ in 0..2 {
        alice
            .send_command::<LiveKitModule>(
                LiveKitCommand::EnableMicrophoneRestrictions(UnrestrictedParticipants {
                    unrestricted_participants: BTreeSet::new(),
                }),
                None,
            )
            .await
            .unwrap();

        // wait till the command succeeded
        let success_event = alice.receive_event::<LiveKitModule>().await.unwrap();
        assert_eq!(
            success_event.content,
            LiveKitEvent::MicrophoneRestrictionsEnabled(UnrestrictedParticipants {
                unrestricted_participants: BTreeSet::new()
            })
        );
    }
}
