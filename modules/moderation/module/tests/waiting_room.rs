// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::BTreeSet;

use opentalk_roomserver_module_moderation::ModerationModule;
use opentalk_roomserver_room::mocking::{
    participant::{MockParticipantJoined, MockParticipantWaiting, bob_public_user_profile},
    room::{TestRoom, flush_connected_events},
};
use opentalk_roomserver_types::{
    connection_id::ConnectionId,
    core::{CoreCommand, CoreEvent, LeftWaitingRoom},
    disconnect_reason::DisconnectReason,
};
use opentalk_roomserver_types_moderation::{
    command::ModerationCommand,
    event::{ModerationError, ModerationEvent},
    state::{ModerationState, WaitingParticipantPeerData},
};
use opentalk_types_signaling::ParticipantId;

#[test_log::test(tokio::test)]
async fn join_info() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(true)
        .spawn();
    let bob = room.waiting_room_bob(0).await;
    let alice = room.join_alice_moderator(0).await;

    let moderator_data = alice
        .join_success()
        .module_data
        .get::<ModerationState>()
        .expect("Module data must be present")
        .expect("Module data must not be none")
        .moderator_data
        .expect("Moderator data must be present");
    assert!(moderator_data.waiting_room_enabled);

    let waiting_bob = &moderator_data.waiting_room_participants[0];
    assert!(matches!(
        waiting_bob,
        WaitingParticipantPeerData { participant_id, accepted, .. }
            if *participant_id == bob.id() && !accepted
    ));
}

async fn accept_participant(
    moderator: &mut MockParticipantJoined,
    mut joinee: MockParticipantWaiting,
) -> MockParticipantJoined {
    let event = moderator.receive::<CoreEvent>().await.unwrap();
    assert!(matches!(
        event.payload,
        CoreEvent::JoinedWaitingRoom { participant_id, .. } if participant_id == joinee.id()
    ));
    assert!(joinee.received_nothing());

    moderator
        .send_command::<ModerationModule>(
            ModerationCommand::Accept {
                target: joinee.id(),
            },
            None,
        )
        .await
        .unwrap();
    let event = moderator
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::ParticipantAccepted {
            participant_id: joinee.id()
        }
    );

    let event = joinee
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, ModerationEvent::Accepted);

    joinee
        .send_core_command(CoreCommand::EnterRoom, None)
        .await
        .unwrap();
    let mut joinee = joinee.join_success().await.unwrap();

    let event = moderator.receive::<CoreEvent>().await.unwrap();
    assert!(matches!(
        event.payload,
        CoreEvent::LeftWaitingRoom(
            LeftWaitingRoom { id, connection_id }
        ) if joinee.id() == id && joinee.connection_id() == connection_id
    ));
    let event = moderator.receive::<CoreEvent>().await.unwrap();
    assert!(matches!(
        event.payload,
        CoreEvent::ParticipantConnected {
            participant_id,
            connection_id,
            ..
            } if participant_id == joinee.id() && connection_id == joinee.connection_id()
    ));
    assert!(moderator.received_nothing());
    assert!(joinee.received_nothing());
    joinee
}

/// 1. Spawn room with activated waiting room
/// 2. Alice joins as a moderator
/// 3. Charlie joins via the waiting room (Test participant to verify events are send out correctly)
/// 4. Bob joins twice and is put in the waiting room (multiple connections)
/// 5. Alice accepts bob
/// 6. Both of Bobs devices join the room
/// 7. Moderators receive LeftWaitingRoom event; Normal users receive Joined event
#[test_log::test(tokio::test)]
async fn join_via_waiting_room() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(true)
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let charlie = room.waiting_room_charlie(0).await;
    let mut charlie = accept_participant(&mut alice, charlie).await;

    let mut bob_0 = room.waiting_room_bob(0).await;
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert!(matches!(
        event.payload,
        CoreEvent::JoinedWaitingRoom { participant_id, .. } if participant_id == bob_0.id()
    ));

    let mut bob_1 = room.waiting_room_bob(1).await;
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert!(matches!(
        event.payload,
        CoreEvent::JoinedWaitingRoom{ participant_id, .. } if participant_id == bob_1.id()
    ));

    assert!(bob_0.received_nothing());
    assert!(bob_1.received_nothing());
    alice
        .send_command::<ModerationModule>(ModerationCommand::Accept { target: bob_0.id() }, None)
        .await
        .unwrap();
    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::ParticipantAccepted {
            participant_id: bob_0.id()
        }
    );

    let event = bob_0
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, ModerationEvent::Accepted);

    let event = bob_1
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, ModerationEvent::Accepted);

    bob_0
        .send_core_command(CoreCommand::EnterRoom, None)
        .await
        .unwrap();
    let bob_0 = bob_0.join_success().await.unwrap();
    let bob_1 = bob_1.join_success().await.unwrap();

    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert!(matches!(
        event.payload,
        CoreEvent::LeftWaitingRoom(
            LeftWaitingRoom { id, connection_id }
        ) if bob_0.id() == id && bob_0.connection_id() == connection_id
    ));
    // charlie should only receive the JoinedEvent, which will be checked next.

    async fn receive_joined_events(
        participant: &mut MockParticipantJoined,
        expected: BTreeSet<(ParticipantId, ConnectionId)>,
    ) {
        let mut joined_connections = BTreeSet::new();
        for _ in 0..2 {
            let event = participant.receive::<CoreEvent>().await.unwrap();
            match event.payload {
                CoreEvent::ParticipantConnected {
                    participant_id,
                    connection_id,
                    ..
                } => {
                    joined_connections.insert((participant_id, connection_id));
                }
                other => {
                    panic!("Unexpected CoreEvent: {other:?}");
                }
            }
        }
        assert_eq!(
            joined_connections,
            expected,
            "Participant {} didn't receive all joined events",
            participant.display_name()
        );
    }
    for p in [&mut alice, &mut charlie] {
        receive_joined_events(
            p,
            BTreeSet::from([
                (bob_0.id(), bob_0.connection_id()),
                (bob_1.id(), bob_1.connection_id()),
            ]),
        )
        .await;
    }

    // No additional messages, bob_0 or bob_1 might receive a ParticipantJoined event depending on
    // the order of joining
    assert!(alice.received_nothing());
    assert!(charlie.received_nothing());
}

/// 1. Spawn room with activated waiting room
/// 2. Alice joins as a moderator
/// 3. Alice accepts bob
#[test_log::test(tokio::test)]
async fn accept_unknown_participant() {
    let mut room = TestRoom::builder()
        .waiting_room(true)
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::Accept {
                target: ParticipantId::from_u128(12),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice.receive::<ModerationEvent>().await.unwrap();
    assert!(
        matches!(
            event.payload,
            ModerationEvent::Error(ModerationError::NotWaiting)
        ),
        "Expected moderation error, got: {:?}",
        event.payload
    );
}

/// 1. Spawn room with activated waiting room
/// 2. Alice joins as a moderator
#[test_log::test(tokio::test)]
async fn moderators_skip_waiting_room() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(true)
        .spawn();
    room.join_alice_moderator(0).await;
}

/// 1. Spawn room with activated waiting room
/// 2. Alice joins as a moderator
/// 3. Bob joins and is accepted
/// 4. Bob leaves
/// 5. When Bob joins again, he skips the waiting room
#[test_log::test(tokio::test)]
async fn registered_users_once_accepted_always_skip() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(true)
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    let bob = room.waiting_room_bob(0).await;
    let bob = accept_participant(&mut alice, bob).await;

    bob.disconnect().await.unwrap();
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert!(matches!(
        event.payload,
        CoreEvent::ParticipantDisconnected { .. }
    ));
    room.join_bob(0).await;
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert!(matches!(
        event.payload,
        CoreEvent::ParticipantConnected { .. }
    ));
}

/// 1. Spawn room with activated waiting room
/// 2. Alice joins as a moderator
/// 3. Gustav joins and is accepted
/// 4. Gustav leaves
/// 5. When Gustav joins again, he skips the waiting room
#[test_log::test(tokio::test)]
async fn guest_users_once_accepted_always_skip() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(true)
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    let gustav = room.waiting_room_gustav_guest().await;
    let gustav = accept_participant(&mut alice, gustav).await;

    gustav.disconnect().await.unwrap();
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert!(matches!(
        event.payload,
        CoreEvent::ParticipantDisconnected { .. }
    ));
    room.join_gustav_guest().await;
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert!(matches!(
        event.payload,
        CoreEvent::ParticipantConnected { .. }
    ));
}

/// 1. Spawn room with activated waiting room
/// 2. Alice joins the room as a moderator
/// 3. Bob enters the waiting room (Alice receives notification)
/// 4. Bob leaves the waiting room (Alice receives notification)
#[test_log::test(tokio::test)]
async fn event_when_leaving_waiting_room() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(true)
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    let bob = room.waiting_room_bob(0).await;
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert!(matches!(event.payload, CoreEvent::JoinedWaitingRoom { .. }));

    bob.disconnect().await.unwrap();
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert!(matches!(event.payload, CoreEvent::LeftWaitingRoom(..)));
}

#[test_log::test(tokio::test)]
async fn enable_waiting_room() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<ModerationModule>(ModerationCommand::EnableWaitingRoom, None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, ModerationEvent::WaitingRoomEnabled);

    // Sending the command again produces the same event
    alice
        .send_command::<ModerationModule>(ModerationCommand::EnableWaitingRoom, None)
        .await
        .unwrap();
    assert_eq!(event, ModerationEvent::WaitingRoomEnabled);
}

#[test_log::test(tokio::test)]
async fn enable_waiting_room_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut bob = room.join_bob(0).await;

    // Bob tries to enable the waiting room
    bob.send_command::<ModerationModule>(ModerationCommand::EnableWaitingRoom, None)
        .await
        .unwrap();

    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;

    // Bob is not allowed to enable the waiting room because he isn't a moderator
    assert_eq!(
        event,
        ModerationEvent::Error(ModerationError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn disable_waiting_room() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(true)
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<ModerationModule>(ModerationCommand::DisableWaitingRoom, None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, ModerationEvent::WaitingRoomDisabled);
}

#[test_log::test(tokio::test)]
async fn disable_waiting_room_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice enables the waiting room
    alice
        .send_command::<ModerationModule>(ModerationCommand::EnableWaitingRoom, None)
        .await
        .unwrap();

    // Alice receives the waiting room enabled event
    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, ModerationEvent::WaitingRoomEnabled);

    // Bob receives the waiting room enabled event
    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, ModerationEvent::WaitingRoomEnabled);

    // Bob tries to disable the waiting room
    bob.send_command::<ModerationModule>(ModerationCommand::DisableWaitingRoom, None)
        .await
        .unwrap();
    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;

    // Bob is not allowed to disable the waiting room because he isn't a moderator
    assert_eq!(
        event,
        ModerationEvent::Error(ModerationError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn send_to_waiting_room_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<ModerationModule>(ModerationCommand::EnableWaitingRoom, None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, ModerationEvent::WaitingRoomEnabled);

    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, ModerationEvent::WaitingRoomEnabled);

    bob.send_command::<ModerationModule>(
        ModerationCommand::SendToWaitingRoom { target: alice.id() },
        None,
    )
    .await
    .unwrap();

    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::Error(ModerationError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn cannot_send_owner_to_waiting_room() {
    let mut room = TestRoom::builder()
        .owner(bob_public_user_profile())
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::SendToWaitingRoom { target: bob.id() },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::Error(ModerationError::CannotSendRoomOwnerToWaitingRoom)
    );
}

#[test_log::test(tokio::test)]
async fn send_to_waiting_room() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(false) // waiting room is enabled automatically by move
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::SendToWaitingRoom { target: bob.id() },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, ModerationEvent::WaitingRoomEnabled);

    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, ModerationEvent::WaitingRoomEnabled);

    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert!(matches!(
        event,
        CoreEvent::ParticipantDisconnected {
            participant_id,
            connection_id,
            reason,
        } if participant_id == bob.id() && connection_id == bob.connection_id() && reason == DisconnectReason::SentToWaitingRoom
    ));

    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    let CoreEvent::JoinedWaitingRoom {
        participant_id,
        connection_ids,
        ..
    } = event
    else {
        panic!("Expected JoinedWaitingRoom event, got: {:?}", event);
    };
    assert_eq!(participant_id, bob.id());
    assert_eq!(connection_ids, vec![bob.connection_id()]);

    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, ModerationEvent::SentToWaitingRoom);

    // Bob does not receive the JoinedWaitingRoom event
    assert!(bob.received_nothing());
}
