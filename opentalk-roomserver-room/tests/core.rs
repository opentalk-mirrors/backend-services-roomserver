// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use insta::assert_json_snapshot;
use opentalk_roomserver_room::{
    mocking::{mock_module::MockModule, room::TestRoom},
    signaling::CORE_MODULES,
};
use opentalk_roomserver_types::core::{CoreCommand, CoreError, CoreEvent};

#[test_log::test(tokio::test)]
async fn join_success() {
    let mut room = TestRoom::builder().register_module::<MockModule>().spawn();

    // Alice joins
    let alice = room.join_alice_moderator(0).await;

    let join_success = alice.join_success();
    assert_json_snapshot!(join_success, {
        ".connection_id" => "[uuid]",
        ".device_id" => "[uuid]",
    }, @r#"
    {
      "id": "00000000-0000-0000-0000-0000000a11ce",
      "connection_id": "[uuid]",
      "device_id": "[uuid]",
      "connections": [],
      "display_name": "Alice the angry",
      "avatar_url": "https://example.com/avatar-of-alice",
      "role": "moderator",
      "tariff": {
        "id": "00000000-0000-0000-0000-000000000001",
        "name": "Default Tariff",
        "quotas": {},
        "used_quota": {},
        "disabled_features": []
      },
      "enabled_modules": [
        "mock",
        "core",
        "breakout"
      ],
      "module_data": {
        "mock": "Self: 00000000-0000-0000-0000-0000000a11ce"
      },
      "participants": [],
      "event_info": null,
      "meeting_details": {
        "streaming_links": []
      },
      "room_info": {
        "id": "00000000-0000-0000-0000-000000000001",
        "created_by": {
          "title": "M.Sc.",
          "firstname": "Alice",
          "lastname": "Aal",
          "display_name": "Alice the angry",
          "avatar_url": "https://example.com/avatar-of-alice"
        }
      },
      "is_room_owner": true
    }
    "#);
}

#[test_log::test(tokio::test)]
async fn participant_joined() {
    let mut room = TestRoom::builder().register_module::<MockModule>().spawn();

    // Alice joins
    let mut alice = room.join_alice_moderator(0).await;
    let bob = room.join_bob(0).await;

    assert_json_snapshot!(bob.join_success(), {
      ".connection_id" => "[uuid]",
      ".device_id" => "[uuid]",
      ".participants[0].connections[0].connection_id" => "[uuid]",
      ".participants[0].connections[0].device_id" => "[uuid]",
      ".participants[0].module_data.core.joined_at" => "[timestamp]",
    }, @r#"
    {
      "id": "00000000-0000-0000-0000-000000000b0b",
      "connection_id": "[uuid]",
      "device_id": "[uuid]",
      "connections": [],
      "display_name": "Bob the bold",
      "avatar_url": "https://example.com/avatar-of-bob",
      "role": "user",
      "tariff": {
        "id": "00000000-0000-0000-0000-000000000001",
        "name": "Default Tariff",
        "quotas": {},
        "used_quota": {},
        "disabled_features": []
      },
      "enabled_modules": [
        "mock",
        "core",
        "breakout"
      ],
      "module_data": {
        "mock": "Self: 00000000-0000-0000-0000-000000000b0b"
      },
      "participants": [
        {
          "id": "00000000-0000-0000-0000-0000000a11ce",
          "connections": [
            {
              "connection_id": "[uuid]",
              "device_id": "[uuid]"
            }
          ],
          "module_data": {
            "breakout": {
              "room": {
                "kind": "main"
              }
            },
            "core": {
              "avatar_url": "https://example.com/avatar-of-alice",
              "display_name": "Alice the angry",
              "is_room_owner": true,
              "joined_at": "[timestamp]",
              "participation_kind": "registered",
              "role": "moderator"
            },
            "mock": "About 00000000-0000-0000-0000-0000000a11ce for 00000000-0000-0000-0000-000000000b0b"
          }
        }
      ],
      "event_info": null,
      "meeting_details": {
        "streaming_links": []
      },
      "room_info": {
        "id": "00000000-0000-0000-0000-000000000001",
        "created_by": {
          "title": "M.Sc.",
          "firstname": "Alice",
          "lastname": "Aal",
          "display_name": "Alice the angry",
          "avatar_url": "https://example.com/avatar-of-alice"
        }
      },
      "is_room_owner": false
    }
    "#);

    let bob_joined = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert_json_snapshot!(bob_joined, {
        ".connection_id" => "[uuid]",
        ".peer_data.core.joined_at" => "[timestamp]",
    }, @r#"
    {
      "message": "participant_connected",
      "participant_id": "00000000-0000-0000-0000-000000000b0b",
      "connection_id": "[uuid]",
      "peer_data": {
        "breakout": {
          "room": {
            "kind": "main"
          }
        },
        "core": {
          "avatar_url": "https://example.com/avatar-of-bob",
          "display_name": "Bob the bold",
          "is_room_owner": false,
          "joined_at": "[timestamp]",
          "participation_kind": "registered",
          "role": "user"
        },
        "mock": "From 00000000-0000-0000-0000-000000000b0b for 00000000-0000-0000-0000-0000000a11ce"
      }
    }
    "#
        );
}

#[test_log::test(tokio::test)]
async fn already_in_room() {
    let mut room = TestRoom::builder().register_module::<MockModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_core_command(CoreCommand::EnterRoom, None)
        .await
        .unwrap();

    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert!(matches!(event, CoreEvent::Error(CoreError::AlreadyInRoom)));
}

#[test_log::test(tokio::test)]
async fn recorder_skips_waiting_room() {
    let mut room = TestRoom::builder().waiting_room(true).spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let recorder = room.join_recorder().await;

    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert!(
        matches!(event, CoreEvent::ParticipantConnected { participant_id, connection_id, .. }
            if participant_id == recorder.id() && connection_id == recorder.connection_id())
    );
}

#[test_log::test(tokio::test)]
async fn join_success_contains_core_modules() {
    let mut room = TestRoom::builder().spawn();
    let alice = room.join_alice_moderator(0).await;

    let join_success = alice.join_success();
    assert!(
        CORE_MODULES
            .iter()
            .all(|id| join_success.enabled_modules.contains(id))
    );
}
