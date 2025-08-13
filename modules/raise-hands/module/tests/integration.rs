// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::BTreeSet;

use opentalk_roomserver_module_raise_hands::RaiseHandsModule;
use opentalk_roomserver_room::mocking::{
    participant::MockParticipantJoined,
    room::{TestRoom, flush_connected_events},
};
use opentalk_roomserver_types::{
    breakout::{
        breakout_config::{BreakoutConfig, BreakoutRoomConfig},
        breakout_id::BreakoutId,
        command::BreakoutCommand,
        event::BreakoutEvent,
    },
    room_kind::RoomKind,
};
use opentalk_roomserver_types_raise_hands::{
    command::{RaiseHandsCommand, ResetRaisedHands},
    event::{RaiseHandsError, RaiseHandsEvent},
    state::{RaiseHandsState, RaisedHandState},
};
use pretty_assertions::assert_eq;

#[test_log::test(tokio::test)]
async fn join_success() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    let state = alice
        .join_success()
        .get_module::<RaiseHandsState>()
        .expect("Module data must be present")
        .expect("Moderation module state must not be None");
    assert!(state.raise_hands_enabled);
    assert_eq!(state.raised_hands, Some(BTreeSet::new()));

    alice
        .send_command::<RaiseHandsModule>(RaiseHandsCommand::RaiseHand, None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::HandRaised {
            participant: alice.id()
        }
    );

    let bob = room.join_bob(0).await;
    let state = bob
        .join_success()
        .get_module::<RaiseHandsState>()
        .expect("Module data must be present")
        .expect("Moderation module state must not be None");
    assert!(state.raise_hands_enabled);
    let raised_hands = state.raised_hands.unwrap();
    assert_eq!(raised_hands.len(), 1);
    assert!(matches!(
        raised_hands.first().unwrap(),
        RaisedHandState { participant_id, .. } if *participant_id == alice.id()
    ))
}

#[test_log::test(tokio::test)]
async fn enable_raise_hands_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;

    // Raised hands is enabled by default
    alice
        .send_command::<RaiseHandsModule>(RaiseHandsCommand::DisableRaiseHands, None)
        .await
        .unwrap();

    let event = bob
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaiseHandsDisabled {
            issued_by: alice.id()
        }
    );

    bob.send_command::<RaiseHandsModule>(RaiseHandsCommand::EnableRaiseHands, None)
        .await
        .unwrap();

    let event = bob
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::Error(RaiseHandsError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn disable_raise_hands_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut bob = room.join_bob(0).await;

    bob.send_command::<RaiseHandsModule>(RaiseHandsCommand::DisableRaiseHands, None)
        .await
        .unwrap();

    let event = bob
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::Error(RaiseHandsError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn reset_raise_hands_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut bob = room.join_bob(0).await;

    bob.send_command::<RaiseHandsModule>(
        RaiseHandsCommand::ResetRaisedHands(ResetRaisedHands { target: None }),
        None,
    )
    .await
    .unwrap();

    let event = bob
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::Error(RaiseHandsError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn raise_hand_disabled() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Disable raise hands
    alice
        .send_command::<RaiseHandsModule>(RaiseHandsCommand::DisableRaiseHands, None)
        .await
        .unwrap();

    let event = bob
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaiseHandsDisabled {
            issued_by: alice.id()
        }
    );

    // Bob tries to raise his hand
    bob.send_command::<RaiseHandsModule>(RaiseHandsCommand::RaiseHand, None)
        .await
        .unwrap();

    let event = bob
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::Error(RaiseHandsError::RaiseHandsDisabled)
    );
}

#[test_log::test(tokio::test)]
async fn enable_raise_hands() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<RaiseHandsModule>(RaiseHandsCommand::EnableRaiseHands, None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaiseHandsEnabled {
            issued_by: alice.id()
        }
    );

    let event = bob
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaiseHandsEnabled {
            issued_by: alice.id()
        }
    );
}

#[test_log::test(tokio::test)]
async fn disable_raise_hands() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<RaiseHandsModule>(RaiseHandsCommand::DisableRaiseHands, None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaiseHandsDisabled {
            issued_by: alice.id()
        }
    );

    let event = bob
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaiseHandsDisabled {
            issued_by: alice.id()
        }
    );
}

#[test_log::test(tokio::test)]
async fn raise_and_lower_hand() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    // Bob raises his hand
    raise_hand(&mut bob, &mut [&mut alice, &mut charlie]).await;
    // Bob lowers his hand
    lower_hand(&mut bob, &mut [&mut alice, &mut charlie]).await;
}

#[test_log::test(tokio::test)]
async fn reset_raised_hands_partial() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    // Bob and Charlie raise their hands
    raise_hand(&mut bob, &mut [&mut alice, &mut charlie]).await;
    raise_hand(&mut charlie, &mut [&mut alice, &mut bob]).await;

    // Alice resets raised hands for Bob
    alice
        .send_command::<RaiseHandsModule>(
            RaiseHandsCommand::ResetRaisedHands(ResetRaisedHands {
                target: Some(BTreeSet::from_iter([bob.id()])),
            }),
            None,
        )
        .await
        .unwrap();

    // Everyone receives the reset event
    let event = alice
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaisedHandResetByModerator {
            issued_by: alice.id(),
            participants: BTreeSet::from_iter([bob.id()])
        }
    );

    let event = bob
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaisedHandResetByModerator {
            issued_by: alice.id(),
            participants: BTreeSet::from_iter([bob.id()])
        }
    );

    let event = charlie
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaisedHandResetByModerator {
            issued_by: alice.id(),
            participants: BTreeSet::from_iter([bob.id()])
        }
    );
}

#[test_log::test(tokio::test)]
async fn reset_raised_hands_all() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    // Bob and Charlie raise their hands
    raise_hand(&mut bob, &mut [&mut alice, &mut charlie]).await;
    raise_hand(&mut charlie, &mut [&mut alice, &mut bob]).await;

    // Alice resets raised hands for all
    alice
        .send_command::<RaiseHandsModule>(
            RaiseHandsCommand::ResetRaisedHands(ResetRaisedHands { target: None }),
            None,
        )
        .await
        .unwrap();

    // Everyone receives the reset event
    let event = alice
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaisedHandResetByModerator {
            issued_by: alice.id(),
            participants: BTreeSet::from_iter([bob.id(), charlie.id()])
        }
    );

    let event = bob
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaisedHandResetByModerator {
            issued_by: alice.id(),
            participants: BTreeSet::from_iter([bob.id(), charlie.id()])
        }
    );

    let event = charlie
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaisedHandResetByModerator {
            issued_by: alice.id(),
            participants: BTreeSet::from_iter([bob.id(), charlie.id()])
        }
    );
}

#[test_log::test(tokio::test)]
async fn reset_raise_hands_ignores_unraised() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    // Bob raises his hands
    raise_hand(&mut bob, &mut [&mut alice, &mut charlie]).await;

    // Alice resets raised hands all
    alice
        .send_command::<RaiseHandsModule>(
            RaiseHandsCommand::ResetRaisedHands(ResetRaisedHands { target: None }),
            None,
        )
        .await
        .unwrap();

    // Everyone receives the reset event
    let event = alice
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaisedHandResetByModerator {
            issued_by: alice.id(),
            participants: BTreeSet::from_iter([bob.id()])
        }
    );

    let event = bob
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaisedHandResetByModerator {
            issued_by: alice.id(),
            participants: BTreeSet::from_iter([bob.id()])
        }
    );

    let event = charlie
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaisedHandResetByModerator {
            issued_by: alice.id(),
            participants: BTreeSet::from_iter([bob.id()])
        }
    );
}

#[test_log::test(tokio::test)]
async fn reset_raised_hands_noop() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<RaiseHandsModule>(
            RaiseHandsCommand::ResetRaisedHands(ResetRaisedHands { target: None }),
            None,
        )
        .await
        .unwrap();

    // Resetting raised hands when no hands were raised does not produce any events
    assert!(alice.received_nothing());
    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
async fn enable_raise_hands_does_not_overwrite() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice raises her hand
    raise_hand(&mut alice, &mut []).await;

    // Alice sends the enable raise hands command
    alice
        .send_command::<RaiseHandsModule>(RaiseHandsCommand::EnableRaiseHands, None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaiseHandsEnabled {
            issued_by: alice.id()
        }
    );

    let bob = room.join_bob(0).await;
    let state = bob
        .join_success()
        .get_module::<RaiseHandsState>()
        .expect("Module data must be present")
        .expect("Moderation module state must not be None");
    // The raised hands were not affected by the enable command
    assert!(matches!(state.raised_hands.unwrap().first().unwrap(),
        RaisedHandState { participant_id, .. } if *participant_id == alice.id()));
}

#[test_log::test(tokio::test)]
async fn breakout_switch() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_breakout_command(
            BreakoutCommand::Start(BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "Breakout Room".into(),
                    assignments: Vec::new(),
                }],
                duration: None,
            }),
            None,
        )
        .await
        .unwrap();

    let event = alice.receive::<BreakoutEvent>().await.unwrap().payload;
    assert!(matches!(event, BreakoutEvent::Started { .. }));

    let event = bob.receive::<BreakoutEvent>().await.unwrap().payload;
    assert!(matches!(event, BreakoutEvent::Started { .. }));

    alice
        .send_breakout_command(
            BreakoutCommand::SwitchRoom(RoomKind::Breakout(BreakoutId::from(0))),
            None,
        )
        .await
        .unwrap();

    let event = alice.receive::<BreakoutEvent>().await.unwrap().payload;
    let BreakoutEvent::SwitchedRoom { module_data, .. } = event else {
        panic!("Received unexpected event: {:?}", event);
    };
    let state = module_data
        .get::<RaiseHandsState>()
        .expect("Module data must be present")
        .expect("Moderation module state must not be None");
    assert!(state.raise_hands_enabled);
    assert_eq!(state.raised_hands, Some(BTreeSet::new()));

    let event = bob.receive::<BreakoutEvent>().await.unwrap().payload;
    assert!(matches!(
        event,
        BreakoutEvent::ParticipantSwitchedRoom { .. }
    ));

    // Alice raises her hand in the breakout room
    alice
        .send_command::<RaiseHandsModule>(RaiseHandsCommand::RaiseHand, None)
        .await
        .unwrap();

    assert!(bob.received_nothing());

    bob.send_breakout_command(
        BreakoutCommand::SwitchRoom(RoomKind::Breakout(BreakoutId::from(0))),
        None,
    )
    .await
    .unwrap();

    let event = bob.receive::<BreakoutEvent>().await.unwrap().payload;
    let BreakoutEvent::SwitchedRoom { module_data, .. } = event else {
        panic!("Received unexpected event: {:?}", event);
    };

    let state = module_data
        .get::<RaiseHandsState>()
        .expect("Module data must be present")
        .expect("Moderation module state must not be None");
    assert!(state.raise_hands_enabled);
    let raised_hands = state.raised_hands.unwrap();
    assert_eq!(raised_hands.len(), 1);
    assert!(matches!(
        raised_hands.first().unwrap(),
        RaisedHandState { participant_id, .. } if *participant_id == alice.id()
    ));
}

#[test_log::test(tokio::test)]
async fn raise_hand_is_breakout_local() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    alice
        .send_breakout_command(
            BreakoutCommand::Start(BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "Breakout Room".into(),
                    assignments: Vec::new(),
                }],
                duration: None,
            }),
            None,
        )
        .await
        .unwrap();

    let event = alice.receive::<BreakoutEvent>().await.unwrap().payload;
    assert!(matches!(event, BreakoutEvent::Started { .. }));

    let event = bob.receive::<BreakoutEvent>().await.unwrap().payload;
    assert!(matches!(event, BreakoutEvent::Started { .. }));

    let event = charlie.receive::<BreakoutEvent>().await.unwrap().payload;
    assert!(matches!(event, BreakoutEvent::Started { .. }));

    // Charlie switches to the breakout room
    charlie
        .send_breakout_command(
            BreakoutCommand::SwitchRoom(RoomKind::Breakout(BreakoutId::from(0))),
            None,
        )
        .await
        .unwrap();

    let event = alice.receive::<BreakoutEvent>().await.unwrap().payload;
    assert!(matches!(
        event,
        BreakoutEvent::ParticipantSwitchedRoom { .. }
    ));

    let event = bob.receive::<BreakoutEvent>().await.unwrap().payload;
    assert!(matches!(
        event,
        BreakoutEvent::ParticipantSwitchedRoom { .. }
    ));

    let event = charlie.receive::<BreakoutEvent>().await.unwrap().payload;
    assert!(matches!(event, BreakoutEvent::SwitchedRoom { .. }));

    // Bob raises his hand in the main room
    raise_hand(&mut bob, &mut [&mut alice]).await;

    // Charlie does not receive the raised hand event
    assert!(charlie.received_nothing());

    // Bob lowers his hand in the main room
    lower_hand(&mut bob, &mut [&mut alice]).await;

    // Charlie does not receive the lowered hand event
    assert!(charlie.received_nothing());

    alice
        .send_command::<RaiseHandsModule>(RaiseHandsCommand::DisableRaiseHands, None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaiseHandsDisabled {
            issued_by: alice.id()
        }
    );

    let event = bob
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaiseHandsDisabled {
            issued_by: alice.id()
        }
    );

    assert!(charlie.received_nothing());

    // Charlie can still raise his hand
    raise_hand(&mut charlie, &mut []).await;
}

async fn raise_hand(
    participant: &mut MockParticipantJoined,
    others: &mut [&mut MockParticipantJoined],
) {
    participant
        .send_command::<RaiseHandsModule>(RaiseHandsCommand::RaiseHand, None)
        .await
        .unwrap();

    let event = participant
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::HandRaised {
            participant: participant.id()
        }
    );

    for other in others {
        let event = other
            .receive_event::<RaiseHandsModule>()
            .await
            .unwrap()
            .payload;
        assert_eq!(
            event,
            RaiseHandsEvent::HandRaised {
                participant: participant.id()
            }
        );
    }
}

async fn lower_hand(
    participant: &mut MockParticipantJoined,
    others: &mut [&mut MockParticipantJoined],
) {
    participant
        .send_command::<RaiseHandsModule>(RaiseHandsCommand::LowerHand, None)
        .await
        .unwrap();

    let event = participant
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::HandLowered {
            participant: participant.id()
        }
    );

    for other in others {
        let event = other
            .receive_event::<RaiseHandsModule>()
            .await
            .unwrap()
            .payload;
        assert_eq!(
            event,
            RaiseHandsEvent::HandLowered {
                participant: participant.id()
            }
        );
    }
}
