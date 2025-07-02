// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_room::mocking::{
    mock_module::{MockCommand, MockModule},
    room::TestRoom,
};
use opentalk_roomserver_types::error::SignalingError;
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
        "content": {
            "invalid": "command"
        }
    });
    alice.send_command_raw(command).await.unwrap();

    let event = alice.receive::<SignalingError>().await.unwrap();
    assert_eq!(event.transaction_id, Some(0));
}
