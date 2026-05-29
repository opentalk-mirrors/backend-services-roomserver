// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::assert_matches;

use opentalk_roomserver_module_moderation::ModerationModule;
use opentalk_roomserver_room::mocking::room::{TestRoom, flush_connected_events};
use opentalk_roomserver_types::{
    client_parameters::Role, core::CoreEvent, disconnect_reason::DisconnectReason,
    kick_reason::KickReason, signaling::websocket::CloseFrame,
};
use opentalk_roomserver_types_moderation::{
    command::ModerationCommand,
    event::{ModerationError, ModerationEvent},
};
use opentalk_types_signaling::ParticipantId;

#[test_log::test(tokio::test)]
async fn insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut bob = room.join_bob(0).await;
    let charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut bob]).await;

    // Bob tries to kick charlie
    bob.send_command::<ModerationModule>(
        ModerationCommand::Kick {
            target: charlie.id(),
        },
        None,
    )
    .await
    .unwrap();

    // Bob receives an error because he is not a moderator
    let event = bob.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::Error(ModerationError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn can_not_kick_room_owner() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    // Alice grants Bob moderator rights
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::UpdateRole {
                participant_id: bob.id(),
                new_role: Role::Moderator,
            },
            None,
        )
        .await
        .unwrap();

    // Bob receives the role updated event
    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::RoleUpdated {
            participant_id: bob.id(),
            new_role: Role::Moderator
        }
    );

    // Alice grants Charlie moderator rights
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::UpdateRole {
                participant_id: charlie.id(),
                new_role: Role::Moderator,
            },
            None,
        )
        .await
        .unwrap();

    // Bob receives the role update event
    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::RoleUpdated {
            participant_id: charlie.id(),
            new_role: Role::Moderator
        }
    );

    // Bob tries to kick Charlie
    bob.send_command::<ModerationModule>(
        ModerationCommand::Kick {
            target: charlie.id(),
        },
        None,
    )
    .await
    .unwrap();

    // Bob receives an error because the room owner can not be kicked
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
async fn unknown_participant() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::Kick {
                target: ParticipantId::from_u128(0xb0b),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::Error(ModerationError::UnknownParticipant)
    );
}

#[test_log::test(tokio::test)]
async fn kick_participant() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<ModerationModule>(ModerationCommand::Kick { target: bob.id() }, None)
        .await
        .unwrap();

    // The waiting room gets enabled
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

    // Bob receives the kicked event
    let event = bob.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::Kicked {
            reason: KickReason::Kicked
        }
    );

    assert_eq!(
        bob.receive_close_frame().await.unwrap(),
        Some(CloseFrame {
            code: 1000,
            reason: "closed by server".to_string(),
        })
    );

    // Alice receives the disconnect event
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert_matches!(
        event.payload,
        CoreEvent::ParticipantDisconnected {
            participant_id,
            reason,
            ..
        } if participant_id == bob.id() && reason == DisconnectReason::Kicked,
    );
}

#[test_log::test(tokio::test)]
async fn cannot_kick_room_owner() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut frank = room.join_frank_moderator(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Frank tries to kick Alice
    frank
        .send_command::<ModerationModule>(ModerationCommand::Kick { target: alice.id() }, None)
        .await
        .unwrap();

    let event = frank
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    // Alice cannot be kicked because she is the room owner
    assert_eq!(
        event,
        ModerationEvent::Error(ModerationError::CannotKickRoomOwner)
    );

    assert!(alice.received_nothing());
}
