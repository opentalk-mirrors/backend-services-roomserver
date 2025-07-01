// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::time::Duration;

use opentalk_roomserver_module_ping::PingModule;
use opentalk_roomserver_room::mocking::room::{TestRoom, flush_connected_events};
use opentalk_roomserver_types_ping::{
    command::PingCommand,
    event::{PingEvent, Replication},
};
use pretty_assertions::assert_eq;

#[test_log::test(tokio::test)]
async fn ping_sends_response_to_all_connections() {
    let mut room = TestRoom::builder().register_module::<PingModule>().spawn();

    let mut alice_1 = room.join_alice_moderator(1).await;
    let mut alice_2 = room.join_alice_moderator(2).await;
    flush_connected_events(&mut [&mut alice_1]).await;

    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice_1, &mut alice_2]).await;

    alice_1
        .send_command::<PingModule>(PingCommand::Ping, None)
        .await
        .unwrap();

    assert_eq!(
        alice_1.receive_event::<PingModule>().await.unwrap().content,
        PingEvent::Pong
    );
    assert_eq!(
        alice_2.receive_event::<PingModule>().await.unwrap().content,
        PingEvent::Pong
    );
    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
async fn async_delayed_pong_is_received() {
    let mut room = TestRoom::builder().register_module::<PingModule>().spawn();

    let mut alice_1 = room.join_alice_moderator(1).await;
    let mut alice_2 = room.join_alice_moderator(2).await;
    flush_connected_events(&mut [&mut alice_1]).await;

    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice_1, &mut alice_2]).await;

    alice_1
        .send_command::<PingModule>(
            PingCommand::AsyncDelayedPing {
                delay: Duration::from_millis(200),
            },
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        alice_1.receive_event::<PingModule>().await.unwrap().content,
        PingEvent::DelayedPong
    );
    assert_eq!(
        alice_2.receive_event::<PingModule>().await.unwrap().content,
        PingEvent::DelayedPong
    );
    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
async fn replicated_ping_is_replicated_to_all_connections() {
    let mut room = TestRoom::builder().register_module::<PingModule>().spawn();

    let mut alice_1 = room.join_alice_moderator(1).await;
    let mut alice_2 = room.join_alice_moderator(2).await;
    flush_connected_events(&mut [&mut alice_1]).await;

    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice_1, &mut alice_2]).await;

    alice_1
        .send_command::<PingModule>(PingCommand::ReplicatedPing, None)
        .await
        .unwrap();
    assert_eq!(
        alice_2.receive_event::<PingModule>().await.unwrap().content,
        PingEvent::Replication(Replication::ReplicatedPing)
    );

    assert_eq!(
        alice_1.receive_event::<PingModule>().await.unwrap().content,
        PingEvent::Pong
    );
    assert_eq!(
        alice_2.receive_event::<PingModule>().await.unwrap().content,
        PingEvent::Pong
    );
    assert!(bob.received_nothing());
}
