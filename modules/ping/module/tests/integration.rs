// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{str::FromStr, time::Duration};

use opentalk_roomserver_module_ping::PingModule;
use opentalk_roomserver_room::mocking::{participant::MockParticipantJoining, room::TestRoom};
use opentalk_roomserver_signaling::signaling_module::SignalingModule;
use opentalk_roomserver_types::{core_event::CoreEvent, error::SignalingError};
use opentalk_roomserver_types_ping::{
    command::PingCommand,
    error::PingError,
    event::{PingEvent, Replication},
};
use opentalk_types_common::roomserver::DeviceSecret;
use pretty_assertions::assert_eq;

#[test_log::test(tokio::test)]
async fn ping_sends_response_to_all_connections() {
    let mut room = TestRoom::builder().register_module::<PingModule>().spawn();

    let mut alice_1 = MockParticipantJoining::alice()
        .secret(DeviceSecret::from_str("Alice Device Secret 1").expect("Valid device secret"))
        .join(&mut room)
        .await
        .unwrap();
    let mut alice_2 = MockParticipantJoining::alice()
        .secret(DeviceSecret::from_str("Alice Device Secret 2").expect("Valid device secret"))
        .join(&mut room)
        .await
        .unwrap();
    alice_1.receive::<CoreEvent>().await.unwrap(); // alice-2 ParticipantJoined event

    let mut bob = room.join_bob().await;
    alice_1.receive::<CoreEvent>().await.unwrap(); // bob ParticipantJoined event
    alice_2.receive::<CoreEvent>().await.unwrap(); // bob ParticipantJoined event

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
async fn blocking_delayed_pong_is_received() {
    let mut room = TestRoom::builder().register_module::<PingModule>().spawn();

    let mut alice_1 = MockParticipantJoining::alice()
        .secret(DeviceSecret::from_str("Alice Device Secret 1").expect("Valid device secret"))
        .join(&mut room)
        .await
        .unwrap();
    let mut alice_2 = MockParticipantJoining::alice()
        .secret(DeviceSecret::from_str("Alice Device Secret 2").expect("Valid device secret"))
        .join(&mut room)
        .await
        .unwrap();
    alice_1.receive::<CoreEvent>().await.unwrap(); // alice-2 ParticipantJoined event

    let mut bob = room.join_bob().await;
    alice_1.receive::<CoreEvent>().await.unwrap(); // bob ParticipantJoined event
    alice_2.receive::<CoreEvent>().await.unwrap(); // bob ParticipantJoined event

    alice_1
        .send_command::<PingModule>(
            PingCommand::BlockingDelayedPing {
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
async fn async_delayed_pong_is_received() {
    let mut room = TestRoom::builder().register_module::<PingModule>().spawn();

    let mut alice_1 = MockParticipantJoining::alice()
        .secret(DeviceSecret::from_str("Alice Device Secret 1").expect("Valid device secret"))
        .join(&mut room)
        .await
        .unwrap();
    let mut alice_2 = MockParticipantJoining::alice()
        .secret(DeviceSecret::from_str("Alice Device Secret 2").expect("Valid device secret"))
        .join(&mut room)
        .await
        .unwrap();
    alice_1.receive::<CoreEvent>().await.unwrap(); // alice-2 ParticipantJoined event

    let mut bob = room.join_bob().await;
    alice_1.receive::<CoreEvent>().await.unwrap(); // bob ParticipantJoined event
    alice_2.receive::<CoreEvent>().await.unwrap(); // bob ParticipantJoined event

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
async fn error_ping_responds_with_error() {
    let mut room = TestRoom::builder().register_module::<PingModule>().spawn();

    let mut alice_1 = MockParticipantJoining::alice()
        .secret(DeviceSecret::from_str("Alice Device Secret 1").expect("Valid device secret"))
        .join(&mut room)
        .await
        .unwrap();
    let mut alice_2 = MockParticipantJoining::alice()
        .secret(DeviceSecret::from_str("Alice Device Secret 2").expect("Valid device secret"))
        .join(&mut room)
        .await
        .unwrap();
    alice_1.receive::<CoreEvent>().await.unwrap(); // alice-2 ParticipantJoined event

    let mut bob = room.join_bob().await;
    alice_1.receive::<CoreEvent>().await.unwrap(); // bob ParticipantJoined event
    alice_2.receive::<CoreEvent>().await.unwrap(); // bob ParticipantJoined event

    alice_1
        .send_command::<PingModule>(PingCommand::PingError, None)
        .await
        .unwrap();

    assert_eq!(
        alice_1.receive_event::<PingModule>().await.unwrap().content,
        PingEvent::Error(PingError)
    );
    assert_eq!(
        alice_2.receive_event::<PingModule>().await.unwrap().content,
        PingEvent::Error(PingError)
    );
    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
async fn broadcast_should_be_received_by_all() {
    let mut room = TestRoom::builder().register_module::<PingModule>().spawn();

    let mut alice_1 = MockParticipantJoining::alice()
        .secret(DeviceSecret::from_str("Alice Device Secret 1").expect("Valid device secret"))
        .join(&mut room)
        .await
        .unwrap();
    let mut alice_2 = MockParticipantJoining::alice()
        .secret(DeviceSecret::from_str("Alice Device Secret 2").expect("Valid device secret"))
        .join(&mut room)
        .await
        .unwrap();
    alice_1.receive::<CoreEvent>().await.unwrap(); // alice-2 ParticipantJoined event

    let mut bob = room.join_bob().await;
    alice_1.receive::<CoreEvent>().await.unwrap(); // bob ParticipantJoined event
    alice_2.receive::<CoreEvent>().await.unwrap(); // bob ParticipantJoined event

    alice_1
        .send_command::<PingModule>(PingCommand::Broadcast, None)
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
    assert_eq!(
        bob.receive_event::<PingModule>().await.unwrap().content,
        PingEvent::Pong
    );
}

#[test_log::test(tokio::test)]
async fn module_should_die() {
    let mut room = TestRoom::builder().register_module::<PingModule>().spawn();

    let mut alice_1 = MockParticipantJoining::alice()
        .secret(DeviceSecret::from_str("Alice Device Secret 1").expect("Valid device secret"))
        .join(&mut room)
        .await
        .unwrap();
    let mut alice_2 = MockParticipantJoining::alice()
        .secret(DeviceSecret::from_str("Alice Device Secret 2").expect("Valid device secret"))
        .join(&mut room)
        .await
        .unwrap();
    alice_1.receive::<CoreEvent>().await.unwrap(); // alice-2 ParticipantJoined event

    let mut bob = room.join_bob().await;
    alice_1.receive::<CoreEvent>().await.unwrap(); // bob ParticipantJoined event
    alice_2.receive::<CoreEvent>().await.unwrap(); // bob ParticipantJoined event

    alice_1
        .send_command::<PingModule>(PingCommand::Die, None)
        .await
        .unwrap();

    assert_eq!(
        alice_1.receive::<SignalingError>().await.unwrap().content,
        SignalingError::FatalModuleError {
            namespace: PingModule::NAMESPACE
        }
    );
    assert_eq!(
        alice_2.receive::<SignalingError>().await.unwrap().content,
        SignalingError::FatalModuleError {
            namespace: PingModule::NAMESPACE
        }
    );
    assert_eq!(
        bob.receive::<SignalingError>().await.unwrap().content,
        SignalingError::FatalModuleError {
            namespace: PingModule::NAMESPACE
        }
    );
}

#[test_log::test(tokio::test)]
async fn replicated_ping_is_replicated_to_all_connections() {
    let mut room = TestRoom::builder().register_module::<PingModule>().spawn();

    let mut alice_1 = MockParticipantJoining::alice()
        .secret(DeviceSecret::from_str("Alice Device Secret 1").expect("Valid device secret"))
        .join(&mut room)
        .await
        .unwrap();
    let mut alice_2 = MockParticipantJoining::alice()
        .secret(DeviceSecret::from_str("Alice Device Secret 2").expect("Valid device secret"))
        .join(&mut room)
        .await
        .unwrap();
    alice_1.receive::<CoreEvent>().await.unwrap(); // alice-2 ParticipantJoined event

    let mut bob = room.join_bob().await;
    alice_1.receive::<CoreEvent>().await.unwrap(); // bob ParticipantJoined event
    alice_2.receive::<CoreEvent>().await.unwrap(); // bob ParticipantJoined event

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
