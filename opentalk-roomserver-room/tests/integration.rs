// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};

use opentalk_roomserver_room::mocking::{
    mock_module::{MockCommand, MockModule},
    participant::{MockParticipantBuilder, bob_public_user_profile},
    room::TestRoom,
};
use opentalk_roomserver_types::{
    client_parameters::Role,
    core::{CoreCommand, CoreError, CoreEvent, RoomCloseReason},
    error::SignalingError,
    rate_limit::RateLimitSettings,
    signaling::websocket::CloseFrame,
    tariff_details::TariffDetails,
};
use opentalk_types_common::{
    roomserver::DeviceSecret,
    tariffs::{QuotaType, TariffId},
};
use serde_json::json;

#[test_log::test(tokio::test)]
async fn response_contains_transaction_id() {
    let mut room = TestRoom::builder().register_module::<MockModule>().spawn();

    // Alice joins
    let mut alice = room.join_alice_moderator(0).await;

    // When no transaction id is sent, the response will not contain one
    alice
        .send_command::<MockModule>(MockCommand::Valid, None)
        .await
        .unwrap();

    let event = alice.receive_event::<MockModule>().await.unwrap();
    assert_eq!(event.transaction_id, None);

    // When a transaction id is sent, the response will contain the same one
    alice
        .send_command::<MockModule>(MockCommand::Valid, Some(0))
        .await
        .unwrap();
    let event = alice.receive_event::<MockModule>().await.unwrap();
    assert_eq!(event.transaction_id, Some(0));
}

#[test_log::test(tokio::test)]
async fn error_contains_transaction_id() {
    let mut room = TestRoom::builder().register_module::<MockModule>().spawn();

    // Alice joins
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<MockModule>(MockCommand::Invalid, Some(0))
        .await
        .unwrap();
    let event = alice.receive_event::<MockModule>().await.unwrap();
    assert_eq!(event.transaction_id, Some(0));
}

#[test_log::test(tokio::test)]
async fn invalid_command_response_contains_transaction_id() {
    let mut room = TestRoom::builder().register_module::<MockModule>().spawn();

    // Alice joins
    let mut alice = room.join_alice_moderator(0).await;

    let command = json!({
        "transaction_id": 0,
        "payload": {
            "invalid": "command"
        }
    });
    alice.send_command_raw(command).await.unwrap();

    let event = alice.receive::<SignalingError>().await.unwrap();
    assert_eq!(event.transaction_id, Some(0));
}

#[test_log::test(tokio::test)]
async fn room_task_time_limit() {
    // Create a room with a tariff that has a time limit of 0 seconds, so it
    // will immediately trigger the time limit quota elapsed event and close the room
    let mut room = TestRoom::builder()
        .tariff(TariffDetails {
            id: TariffId::generate(),
            name: "Immediately closing room".into(),
            quotas: BTreeMap::from_iter([(QuotaType::RoomTimeLimitSecs, 0)]),
            used_quota: BTreeMap::new(),
            disabled_features: BTreeSet::new(),
        })
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert!(matches!(
        event,
        CoreEvent::Closing {
            reason: RoomCloseReason::TimeLimitReached
        }
    ));

    let close_frame = alice.receive_close_frame().await.unwrap();
    assert_eq!(
        close_frame,
        Some(CloseFrame {
            code: 1000,
            reason: "closed by server".to_string(),
        })
    );
}

#[test_log::test(tokio::test)]
async fn participant_receives_same_display_name() {
    let mut room = TestRoom::builder().spawn();
    let bob_0 = room.join_bob(0).await;

    // Bob tries to join with a different display name
    let join_success = MockParticipantBuilder::new(
        bob_public_user_profile(),
        DeviceSecret::from_str("New device secret").unwrap(),
        Role::User,
    )
    .display_name("Bobby".into())
    .join(&mut room)
    .await
    .unwrap();

    // But he receives the same display name as before
    assert_eq!(join_success.display_name(), bob_0.display_name());
}

#[test_log::test(tokio::test)]
async fn exceeding_rate_limit_sends_slow_down() {
    let mut room = TestRoom::builder()
        .ws_rate_limit(RateLimitSettings {
            tokens_per_second: 1,
            token_bucket_size: 1,
        })
        .spawn();

    let mut alice = room.join_alice_moderator(0).await;

    // First command should pass
    // Using CoreCommand because it is available in this module, the actual command does not matter.
    alice
        .send_core_command(CoreCommand::EnterRoom, None)
        .await
        .unwrap();

    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert!(matches!(event, CoreEvent::Error(CoreError::AlreadyInRoom)));

    // Second command should trigger slow down
    alice
        .send_core_command(CoreCommand::EnterRoom, None)
        .await
        .unwrap();

    // Alice receives the slow down event first
    let event = alice.receive::<SignalingError>().await.unwrap().payload;
    assert_eq!(event, SignalingError::SlowDown);

    // Alice receives the actual response to her command afterwards
    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert!(matches!(event, CoreEvent::Error(CoreError::AlreadyInRoom)));
}
