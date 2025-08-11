// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_module_moderation::ModerationModule;
use opentalk_roomserver_room::mocking::room::{TestRoom, flush_connected_events};
use opentalk_roomserver_types_moderation::{
    command::{ChangeDisplayName, ModerationCommand},
    event::{DisplayNameChanged, ModerationError, ModerationEvent},
};
use opentalk_types_common::users::DisplayName;
use opentalk_types_signaling::ParticipantId;

#[test_log::test(tokio::test)]
async fn insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut bob = room.join_bob(0).await;
    let mut gustav = room.join_gustav_guest().await;
    flush_connected_events(&mut [&mut bob]).await;

    // Bob can't change Alice's display name
    bob.send_command::<ModerationModule>(
        ModerationCommand::ChangeDisplayName(ChangeDisplayName {
            new_name: DisplayName::from_str_lossy("Charlie"),
            target: gustav.id(),
        }),
        None,
    )
    .await
    .unwrap();

    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .content;
    assert_eq!(
        event,
        ModerationEvent::Error(ModerationError::InsufficientPermissions)
    );

    assert!(gustav.received_nothing());
}

#[test_log::test(tokio::test)]
async fn display_name_too_short() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut gustav = room.join_gustav_guest().await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::ChangeDisplayName(ChangeDisplayName {
                new_name: DisplayName::from_str_lossy(""),
                target: gustav.id(),
            }),
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .content;
    assert_eq!(
        event,
        ModerationEvent::Error(ModerationError::InvalidDisplayName)
    );

    assert!(gustav.received_nothing());
}

#[test_log::test(tokio::test)]
async fn display_name_too_long() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut gustav = room.join_gustav_guest().await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::ChangeDisplayName(ChangeDisplayName {
                new_name: DisplayName::from_str_lossy(&str::repeat("x", 256)),
                target: gustav.id(),
            }),
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .content;
    assert_eq!(
        event,
        ModerationEvent::Error(ModerationError::InvalidDisplayName)
    );

    assert!(gustav.received_nothing());
}

#[test_log::test(tokio::test)]
async fn unknown_participant() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::ChangeDisplayName(ChangeDisplayName {
                new_name: DisplayName::from_str_lossy("Bob"),
                target: ParticipantId::nil(),
            }),
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .content;
    assert_eq!(
        event,
        ModerationEvent::Error(ModerationError::UnknownParticipant)
    );
}

#[test_log::test(tokio::test)]
async fn cannot_change_name_of_registered_users() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Bob can't change Alice's display name
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::ChangeDisplayName(ChangeDisplayName {
                new_name: DisplayName::from_str_lossy("Charlie"),
                target: bob.id(),
            }),
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .content;
    assert_eq!(
        event,
        ModerationEvent::Error(ModerationError::CannotChangeNameOfRegisteredUsers)
    );

    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
async fn change_display_name() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut gustav = room.join_gustav_guest().await;
    flush_connected_events(&mut [&mut alice]).await;

    // Bob can't change Alice's display name
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::ChangeDisplayName(ChangeDisplayName {
                new_name: DisplayName::from_str_lossy("Bob"),
                target: gustav.id(),
            }),
            None,
        )
        .await
        .unwrap();

    let expected = ModerationEvent::DisplayNameChanged(DisplayNameChanged {
        target: gustav.id(),
        issued_by: alice.id(),
        old_name: DisplayName::from_str_lossy("Gustav the great"),
        new_name: DisplayName::from_str_lossy("Bob"),
    });
    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .content;
    assert_eq!(event, expected.clone());

    let event = gustav
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .content;
    assert_eq!(event, expected);
}
