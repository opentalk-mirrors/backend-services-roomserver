// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_module_timer::TimerModule;
use opentalk_roomserver_room::mocking::{
    participant::{MockParticipant, MockParticipantJoined},
    room::TestRoom,
};
use opentalk_roomserver_types::{
    breakout::{
        breakout_config::{BreakoutConfig, BreakoutRoomConfig},
        breakout_id::BreakoutId,
        command::BreakoutCommand,
        event::BreakoutEvent,
    },
    core_event::CoreEvent,
    room_kind::RoomKind,
};
use opentalk_roomserver_types_timer::{
    command::TimerCommand, error::TimerError, event::TimerEvent,
};
use opentalk_types_signaling_timer::{
    TimerId,
    command::{Kind, Start},
    event::{StopKind, Stopped, UpdatedReadyStatus},
    state::TimerState,
};

async fn create_room() -> (TestRoom, MockParticipantJoined, MockParticipantJoined) {
    let mut room = TestRoom::builder().register_module::<TimerModule>().spawn();
    let mut alice = room.join_alice_moderator().await;
    let bob = room.join_bob().await;

    assert!(matches!(
        alice.receive::<CoreEvent>().await.unwrap().content,
        CoreEvent::ParticipantConnected { .. }
    ));

    (room, alice, bob)
}

#[test_log::test(tokio::test)]
async fn can_not_start_second_timer() {
    let (_room, mut alice, _bob) = create_room().await;

    alice
        .send_command::<TimerModule>(
            TimerCommand::Start(Start {
                kind: Kind::Stopwatch,
                style: None,
                title: None,
                enable_ready_check: false,
            }),
            None,
        )
        .await
        .unwrap();

    assert!(matches!(
        alice.receive_event::<TimerModule>().await.unwrap().content,
        TimerEvent::Started { .. }
    ));

    alice
        .send_command::<TimerModule>(
            TimerCommand::Start(Start {
                kind: Kind::Stopwatch,
                style: None,
                title: None,
                enable_ready_check: false,
            }),
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        alice.receive_event::<TimerModule>().await.unwrap().content,
        TimerEvent::Error(TimerError::TimerAlreadyRunning)
    );
}

#[test_log::test(tokio::test)]
async fn non_moderator_cant_start_timer() {
    let (_room, _alice, mut bob) = create_room().await;

    bob.send_command::<TimerModule>(
        TimerCommand::Start(Start {
            kind: Kind::Countdown { duration: 0 },
            style: None,
            title: None,
            enable_ready_check: false,
        }),
        None,
    )
    .await
    .unwrap();

    assert_eq!(
        bob.receive_event::<TimerModule>().await.unwrap().content,
        TimerEvent::Error(TimerError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn all_participants_receive_timer_events() {
    let (_room, mut alice, mut bob) = create_room().await;

    alice
        .send_command::<TimerModule>(
            TimerCommand::Start(Start {
                kind: Kind::Stopwatch,
                style: None,
                title: None,
                enable_ready_check: false,
            }),
            None,
        )
        .await
        .unwrap();

    assert!(matches!(
        alice.receive_event::<TimerModule>().await.unwrap().content,
        TimerEvent::Started { .. }
    ));

    assert!(matches!(
        bob.receive_event::<TimerModule>().await.unwrap().content,
        TimerEvent::Started { .. }
    ));

    alice
        .send_command::<TimerModule>(TimerCommand::Stop { reason: None }, None)
        .await
        .unwrap();

    assert!(matches!(
        alice.receive_event::<TimerModule>().await.unwrap().content,
        TimerEvent::Stopped(..)
    ));

    assert!(matches!(
        bob.receive_event::<TimerModule>().await.unwrap().content,
        TimerEvent::Stopped(..)
    ));
}

#[test_log::test(tokio::test)]
async fn exceed_timer() {
    let (_room, mut alice, _bob) = create_room().await;

    alice
        .send_command::<TimerModule>(
            TimerCommand::Start(Start {
                kind: Kind::Countdown { duration: 0 },
                style: None,
                title: None,
                enable_ready_check: false,
            }),
            None,
        )
        .await
        .unwrap();

    assert!(matches!(
        alice.receive_event::<TimerModule>().await.unwrap().content,
        TimerEvent::Started { .. }
    ));

    assert!(matches!(
        alice.receive_event::<TimerModule>().await.unwrap().content,
        TimerEvent::Stopped(Stopped {
            kind: StopKind::Expired,
            ..
        })
    ));
}

#[test_log::test(tokio::test)]
async fn stop_timer() {
    let (_room, mut alice, _bob) = create_room().await;

    alice
        .send_command::<TimerModule>(
            TimerCommand::Start(Start {
                kind: Kind::Stopwatch,
                style: None,
                title: None,
                enable_ready_check: false,
            }),
            None,
        )
        .await
        .unwrap();

    assert!(matches!(
        alice.receive_event::<TimerModule>().await.unwrap().content,
        TimerEvent::Started { .. }
    ));

    alice
        .send_command::<TimerModule>(
            TimerCommand::Stop {
                reason: Some("test".into()),
            },
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        alice.receive_event::<TimerModule>().await.unwrap().content,
        TimerEvent::Stopped(Stopped {
            timer_id: TimerId::nil(),
            kind: StopKind::ByModerator(alice.id()),
            reason: Some("test".into())
        })
    );
}

#[test_log::test(tokio::test)]
async fn can_not_update_ready_status_when_timer_not_running() {
    let (_room, mut alice, _bob) = create_room().await;

    alice
        .send_command::<TimerModule>(TimerCommand::UpdateReadyStatus { status: true }, None)
        .await
        .unwrap();

    assert_eq!(
        alice.receive_event::<TimerModule>().await.unwrap().content,
        TimerEvent::Error(TimerError::TimerNotRunning),
    );
}

#[test_log::test(tokio::test)]
async fn can_not_update_ready_status_when_disabled() {
    let (_room, mut alice, _bob) = create_room().await;

    alice
        .send_command::<TimerModule>(
            TimerCommand::Start(Start {
                kind: Kind::Stopwatch,
                style: None,
                title: None,
                enable_ready_check: false,
            }),
            None,
        )
        .await
        .unwrap();

    assert!(matches!(
        alice.receive_event::<TimerModule>().await.unwrap().content,
        TimerEvent::Started { .. }
    ));

    alice
        .send_command::<TimerModule>(TimerCommand::UpdateReadyStatus { status: true }, None)
        .await
        .unwrap();

    assert_eq!(
        alice.receive_event::<TimerModule>().await.unwrap().content,
        TimerEvent::Error(TimerError::ReadyCheckNotEnabled),
    );
}

#[test_log::test(tokio::test)]
async fn update_ready_status() {
    let (_room, mut alice, mut bob) = create_room().await;

    alice
        .send_command::<TimerModule>(
            TimerCommand::Start(Start {
                kind: Kind::Stopwatch,
                style: None,
                title: None,
                enable_ready_check: true,
            }),
            None,
        )
        .await
        .unwrap();

    assert!(matches!(
        alice.receive_event::<TimerModule>().await.unwrap().content,
        TimerEvent::Started { .. }
    ));

    assert!(matches!(
        bob.receive_event::<TimerModule>().await.unwrap().content,
        TimerEvent::Started { .. }
    ));

    bob.send_command::<TimerModule>(TimerCommand::UpdateReadyStatus { status: true }, None)
        .await
        .unwrap();

    assert_eq!(
        alice.receive_event::<TimerModule>().await.unwrap().content,
        TimerEvent::UpdatedReadyStatus(UpdatedReadyStatus {
            participant_id: bob.id(),
            status: true,
            timer_id: TimerId::nil(),
        }),
    );

    assert_eq!(
        bob.receive_event::<TimerModule>().await.unwrap().content,
        TimerEvent::UpdatedReadyStatus(UpdatedReadyStatus {
            participant_id: bob.id(),
            status: true,
            timer_id: TimerId::nil(),
        }),
    );
}

#[test_log::test(tokio::test)]
async fn ready_state_persists() {
    let (mut room, mut alice, _bob) = create_room().await;

    alice
        .send_command::<TimerModule>(
            TimerCommand::Start(Start {
                kind: Kind::Stopwatch,
                style: None,
                title: None,
                enable_ready_check: true,
            }),
            None,
        )
        .await
        .unwrap();

    assert!(matches!(
        alice.receive_event::<TimerModule>().await.unwrap().content,
        TimerEvent::Started { .. }
    ));

    alice
        .send_command::<TimerModule>(TimerCommand::UpdateReadyStatus { status: true }, None)
        .await
        .unwrap();

    assert_eq!(
        alice.receive_event::<TimerModule>().await.unwrap().content,
        TimerEvent::UpdatedReadyStatus(UpdatedReadyStatus {
            participant_id: alice.id(),
            status: true,
            timer_id: TimerId::nil(),
        }),
    );

    alice.disconnect();

    let alice = room.join_alice_moderator().await;

    let timer_state = alice
        .join_success()
        .module_data
        .get::<TimerState>()
        .unwrap()
        .unwrap();

    assert!(timer_state.ready_status.unwrap());
}

#[test_log::test(tokio::test)]
async fn breakout_room_scope() {
    let (mut _room, mut alice, mut bob) = create_room().await;

    // Alice starts breakout rooms with separate rooms for Alice and Bob
    let alice_room = BreakoutRoomConfig {
        name: "alice_room".into(),
        assignments: vec![alice.id()],
    };
    let bob_room = BreakoutRoomConfig {
        name: "bob_room".into(),
        assignments: vec![bob.id()],
    };
    alice
        .send_breakout_command(
            BreakoutCommand::Start(BreakoutConfig {
                rooms: vec![alice_room, bob_room],
                duration: None,
            }),
            None,
        )
        .await
        .unwrap();

    // Alice and Bob receive the BreakoutStarted event
    alice.receive::<BreakoutEvent>().await.unwrap();
    bob.receive::<BreakoutEvent>().await.unwrap();

    // Alice switches to room 0
    switch_room(
        &mut alice,
        &mut bob,
        RoomKind::Breakout(BreakoutId::from(0)),
    )
    .await;

    // Alice starts a timer in her room
    alice
        .send_command::<TimerModule>(
            TimerCommand::Start(Start {
                kind: Kind::Countdown { duration: 0 },
                style: None,
                title: None,
                enable_ready_check: false,
            }),
            None,
        )
        .await
        .unwrap();

    // Alice receives events for the timer
    assert!(matches!(
        alice.receive_event::<TimerModule>().await.unwrap().content,
        TimerEvent::Started { .. }
    ));
    assert!(matches!(
        alice.receive_event::<TimerModule>().await.unwrap().content,
        TimerEvent::Stopped(Stopped {
            kind: StopKind::Expired,
            ..
        })
    ));

    // Bob doesn't
    assert!(bob.received_nothing());
}

async fn switch_room<S>(
    participant: &mut MockParticipant<S>,
    other_participant: &mut MockParticipant<S>,
    room: RoomKind,
) -> BreakoutEvent {
    participant
        .send_breakout_command(BreakoutCommand::SwitchRoom(room), None)
        .await
        .unwrap();
    // All participants receive the ParticipantSwitchedRoom event
    other_participant.receive::<BreakoutEvent>().await.unwrap();
    participant
        .receive::<BreakoutEvent>()
        .await
        .unwrap()
        .content
}

#[test_log::test(tokio::test)]
async fn breakout_room_ready_state() {
    let (mut _room, mut alice, mut bob) = create_room().await;

    // Alice starts breakout rooms with separate rooms for Alice and Bob
    let alice_room = BreakoutRoomConfig {
        name: "alice_room".into(),
        assignments: vec![alice.id()],
    };
    let bob_room = BreakoutRoomConfig {
        name: "bob_room".into(),
        assignments: vec![bob.id()],
    };
    alice
        .send_breakout_command(
            BreakoutCommand::Start(BreakoutConfig {
                rooms: vec![alice_room, bob_room],
                duration: None,
            }),
            None,
        )
        .await
        .unwrap();

    // Alice and Bob receive the BreakoutStarted event
    alice.receive::<BreakoutEvent>().await.unwrap();
    bob.receive::<BreakoutEvent>().await.unwrap();

    // Alice switches to room 0
    switch_room(
        &mut alice,
        &mut bob,
        RoomKind::Breakout(BreakoutId::from(0)),
    )
    .await;

    // Alice starts a timer in room 0
    alice
        .send_command::<TimerModule>(
            TimerCommand::Start(Start {
                kind: Kind::Stopwatch,
                style: None,
                title: None,
                enable_ready_check: true,
            }),
            None,
        )
        .await
        .unwrap();

    // Alice receives events for the timer
    assert!(matches!(
        alice.receive_event::<TimerModule>().await.unwrap().content,
        TimerEvent::Started { .. }
    ));

    // Alice updates her ready state in room 0
    alice
        .send_command::<TimerModule>(TimerCommand::UpdateReadyStatus { status: true }, None)
        .await
        .unwrap();
    assert_eq!(
        alice.receive_event::<TimerModule>().await.unwrap().content,
        TimerEvent::UpdatedReadyStatus(UpdatedReadyStatus {
            participant_id: alice.id(),
            status: true,
            timer_id: TimerId::nil(),
        }),
    );

    // Alice switches to room 1
    switch_room(
        &mut alice,
        &mut bob,
        RoomKind::Breakout(BreakoutId::from(1)),
    )
    .await;

    // Alice switches back to room 0
    let switched_room = switch_room(
        &mut alice,
        &mut bob,
        RoomKind::Breakout(BreakoutId::from(0)),
    )
    .await;

    // Alice is still ready in room 0
    let BreakoutEvent::SwitchedRoom { module_data } = switched_room else {
        unreachable!("Received wrong event");
    };

    let state = module_data.get::<TimerState>().unwrap();
    assert!(matches!(
        state,
        Some(TimerState {
            ready_status: Some(true),
            ..
        })
    ));
}
