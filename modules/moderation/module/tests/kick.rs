// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_module_moderation::ModerationModule;
use opentalk_roomserver_room::mocking::room::{TestRoom, flush_connected_events};
use opentalk_roomserver_types::{core::CoreEvent, disconnect_reason::DisconnectReason};
use opentalk_roomserver_types_moderation::{
    command::{Kick, ModerationCommand},
    event::{ModerationError, ModerationEvent},
};
use opentalk_roomserver_web_api::v1::signaling::websocket::CloseFrame;
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
        ModerationCommand::Kick(Kick {
            target: charlie.id(),
        }),
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
async fn unknown_participant() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::Kick(Kick {
                target: ParticipantId::from_u128(0xb0b),
            }),
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
        .send_command::<ModerationModule>(ModerationCommand::Kick(Kick { target: bob.id() }), None)
        .await
        .unwrap();

    // Bob receives the kicked event
    let event = bob.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(event.payload, ModerationEvent::Kicked);

    assert_eq!(
        bob.receive_close_frame().await.unwrap(),
        Some(CloseFrame {
            code: 1000,
            reason: "closed by server".to_string(),
        })
    );

    // Alice receives the disconnect event
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert!(matches!(
        event.payload,
        CoreEvent::ParticipantDisconnected {
            participant_id,
            reason,
            ..
        } if participant_id == bob.id() && reason == DisconnectReason::Kicked,
    ));
}
