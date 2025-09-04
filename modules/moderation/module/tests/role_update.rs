// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_module_moderation::ModerationModule;
use opentalk_roomserver_module_shared_folder::SharedFolderModule;
use opentalk_roomserver_room::mocking::room::{TestRoom, flush_connected_events};
use opentalk_roomserver_types::{
    client_parameters::Role, core::CoreEvent, disconnect_reason::DisconnectReason,
    room_parameters::EventContext,
};
use opentalk_roomserver_types_moderation::{
    command::ModerationCommand,
    event::{ModerationError, ModerationEvent, RoleUpdate},
};
use opentalk_roomserver_types_shared_folder::event::SharedFolderEvent;
use opentalk_types_common::{
    events::EventId,
    shared_folders::{SharedFolder, SharedFolderAccess},
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

    // Bob tries to change the role of charlie to a moderator
    bob.send_command::<ModerationModule>(
        ModerationCommand::UpdateRole(RoleUpdate {
            participant_id: charlie.id(),
            new_role: Role::Moderator,
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

    // Bob tries to change the role of charlie to a user
    bob.send_command::<ModerationModule>(
        ModerationCommand::UpdateRole(RoleUpdate {
            participant_id: charlie.id(),
            new_role: Role::User,
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
async fn cannot_change_room_owner_role() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut frank = room.join_frank_moderator(0).await;
    // Alice is the room owner for the TestRoom
    let alice = room.join_alice_moderator(0).await;

    flush_connected_events(&mut [&mut frank]).await;

    // Frank tries to change the role of Alice
    frank
        .send_command::<ModerationModule>(
            ModerationCommand::UpdateRole(RoleUpdate {
                participant_id: alice.id(),
                new_role: Role::User,
            }),
            None,
        )
        .await
        .unwrap();

    // Frank receives an error because the room owner cannot have their role changed
    let event = frank.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::Error(ModerationError::CannotChangeRoomOwnerRole)
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
            ModerationCommand::UpdateRole(RoleUpdate {
                participant_id: ParticipantId::from_u128(0xb0b),
                new_role: Role::User,
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
async fn change_role_to_moderator() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let dave = room.join_dave(0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    let role_update = RoleUpdate {
        participant_id: bob.id(),
        new_role: Role::Moderator,
    };

    alice
        .send_command::<ModerationModule>(ModerationCommand::UpdateRole(role_update.clone()), None)
        .await
        .unwrap();

    // Alice and Bob receive the role update event
    let bob_event = bob.receive_event::<ModerationModule>().await.unwrap();
    let alice_event = alice.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(bob_event.payload, ModerationEvent::RoleUpdated(role_update));
    assert_eq!(alice_event.payload, bob_event.payload);

    // Check if bob is now moderator by banning dave
    bob.send_command::<ModerationModule>(ModerationCommand::Ban { target: dave.id() }, None)
        .await
        .unwrap();
    let event = bob.receive::<CoreEvent>().await.unwrap();
    assert!(matches!(
        event.payload,
        CoreEvent::ParticipantDisconnected {
            participant_id,
            reason,
            ..
        }
        if participant_id == dave.id() &&
        reason == DisconnectReason::Banned
    ));
}

#[test_log::test(tokio::test)]
async fn change_role_to_user() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut frank = room.join_frank_moderator(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut dave = room.join_dave(0).await;
    flush_connected_events(&mut [&mut alice, &mut frank]).await;

    let role_update = RoleUpdate {
        participant_id: frank.id(),
        new_role: Role::User,
    };

    alice
        .send_command::<ModerationModule>(ModerationCommand::UpdateRole(role_update.clone()), None)
        .await
        .unwrap();

    // Alice, Frank and Dave receive the role update event
    let bob_event = frank.receive_event::<ModerationModule>().await.unwrap();
    let alice_event = alice.receive_event::<ModerationModule>().await.unwrap();
    let dave_event = dave.receive_event::<ModerationModule>().await.unwrap();

    let expected_event = ModerationEvent::RoleUpdated(role_update);
    assert_eq!(bob_event.payload, expected_event);
    assert_eq!(alice_event.payload, expected_event);
    assert_eq!(dave_event.payload, expected_event);

    // Frank attempts to ban Dave after the role update
    frank
        .send_command::<ModerationModule>(ModerationCommand::Ban { target: dave.id() }, None)
        .await
        .unwrap();
    let event = frank.receive::<ModerationEvent>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::Error(ModerationError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn shared_folder_after_promotion() {
    let shared_folder_config = SharedFolder {
        read: SharedFolderAccess {
            url: "http://opencloud.folder.example.com".to_string(),
            password: "pass".to_string(),
        },
        read_write: Some(SharedFolderAccess {
            url: "http://admin.opencloud.folder.example.com".to_string(),
            password: "admin.pass".to_string(),
        }),
    };

    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .register_module::<SharedFolderModule>()
        .event(EventContext {
            id: EventId::from_u128(1),
            title: "Event 1".parse().unwrap(),
            description: "This is the first event in the shared folder test"
                .parse()
                .unwrap(),
            is_adhoc: false,
            starts_at: None,
            ends_at: None,
            shared_folder: Some(shared_folder_config.clone()),
        })
        .spawn();

    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let role_update = RoleUpdate {
        participant_id: bob.id(),
        new_role: Role::Moderator,
    };

    alice
        .send_command::<ModerationModule>(ModerationCommand::UpdateRole(role_update.clone()), None)
        .await
        .unwrap();

    // Alice and Bob receive the role update event
    let bob_event = bob.receive_event::<ModerationModule>().await.unwrap();
    let alice_event = alice.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(bob_event.payload, ModerationEvent::RoleUpdated(role_update));
    assert_eq!(alice_event.payload, bob_event.payload);

    // Bob receives the new shared folder
    let bob_event = bob.receive_event::<SharedFolderModule>().await.unwrap();

    assert_eq!(
        bob_event.payload,
        SharedFolderEvent::Updated(shared_folder_config)
    )
}

#[test_log::test(tokio::test)]
async fn shared_folder_after_demotion() {
    let shared_folder_config = SharedFolder {
        read: SharedFolderAccess {
            url: "http://opencloud.folder.example.com".to_string(),
            password: "pass".to_string(),
        },
        read_write: Some(SharedFolderAccess {
            url: "http://admin.opencloud.folder.example.com".to_string(),
            password: "admin.pass".to_string(),
        }),
    };

    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .register_module::<SharedFolderModule>()
        .event(EventContext {
            id: EventId::from_u128(1),
            title: "Event 1".parse().unwrap(),
            description: "This is the first event in the shared folder test"
                .parse()
                .unwrap(),
            is_adhoc: false,
            starts_at: None,
            ends_at: None,
            shared_folder: Some(shared_folder_config.clone()),
        })
        .spawn();

    let mut alice = room.join_alice_moderator(0).await;
    let mut frank = room.join_frank_moderator(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let role_update = RoleUpdate {
        participant_id: frank.id(),
        new_role: Role::User,
    };

    alice
        .send_command::<ModerationModule>(ModerationCommand::UpdateRole(role_update.clone()), None)
        .await
        .unwrap();

    // Alice and Bob receive the role update event
    let frank_event = frank.receive_event::<ModerationModule>().await.unwrap();
    let alice_event = alice.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        frank_event.payload,
        ModerationEvent::RoleUpdated(role_update)
    );
    assert_eq!(alice_event.payload, frank_event.payload);

    // Bob receives the new shared folder
    let frank_event = frank.receive_event::<SharedFolderModule>().await.unwrap();

    let read_only = SharedFolder {
        read: shared_folder_config.read,
        read_write: None,
    };

    assert_eq!(frank_event.payload, SharedFolderEvent::Updated(read_only))
}
