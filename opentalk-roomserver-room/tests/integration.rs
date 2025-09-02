// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::BTreeMap;

use opentalk_roomserver_room::mocking::{
    mock_module::{MockCommand, MockModule},
    room::TestRoom,
};
use opentalk_roomserver_types::{core::CoreEvent, error::SignalingError};
use opentalk_roomserver_web_api::v1::signaling::websocket::CloseFrame;
use opentalk_types_common::tariffs::{QuotaType, TariffId, TariffResource};
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
        .tariff(TariffResource {
            id: TariffId::generate(),
            name: "Immediately closing room".into(),
            quotas: BTreeMap::from_iter([(QuotaType::RoomTimeLimitSecs, 0)]),
            modules: BTreeMap::new(),
        })
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert!(matches!(event, CoreEvent::TimeLimitQuotaElapsed));

    let close_frame = alice.receive_close_frame().await.unwrap();
    assert_eq!(
        close_frame,
        Some(CloseFrame {
            code: 1000,
            reason: "closed by server".to_string(),
        })
    );
}
