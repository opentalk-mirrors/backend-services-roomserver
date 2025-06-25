// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_module_chat::ChatModule;
use opentalk_roomserver_room::mocking::{
    participant::MockParticipant,
    room::{TestRoom, flush_connected_events},
};
use opentalk_roomserver_signaling::signaling_module::SignalingModule;
use opentalk_roomserver_types::{
    breakout::{
        breakout_config::{BreakoutConfig, BreakoutRoomConfig},
        breakout_id::BreakoutId,
        command::BreakoutCommand,
        event::BreakoutEvent,
    },
    core_event::CoreEvent,
    join::join_success::JoinSuccess,
    room_kind::RoomKind,
};
use opentalk_roomserver_types_chat::{
    Scope,
    command::{ChatCommand, SendMessage, SetLastSeenTimestamp},
    event::{
        ChatDisabled, ChatEnabled, ChatEvent, Error as ChatError, HistoryCleared, MessageSent,
    },
};
use opentalk_types_common::time::Timestamp;
use opentalk_types_signaling::ParticipantId;
use pretty_assertions::assert_eq;

// This macro shall make comparing messages more readable.
/// Compare the [`MessageSent`] struct with the provided scope, content and source.
/// The macro also ensures that the panic still shows the call site as code location.
/// If a function would be used, the panic would show the function as location regardless
/// of the test it's used in.
macro_rules! assert_message_eq {
    ($expected_scope:expr, $expected_content:expr, $expected_source:expr, $event:expr$(,)?) => {
        if let ChatEvent::MessageSent(msg @ MessageSent { .. }) = $event {
            pretty_assertions::assert_eq!(
                msg,
                &MessageSent {
                    id: msg.id,
                    source: $expected_source,
                    content: $expected_content.to_string(),
                    scope: $expected_scope.clone(),
                },
            );
        } else {
            panic!("Expected ChatEvent::MessageSent, but got: {:?}", $event);
        }
    };
}

/// Once the chat is disabled, messages cannot be sent.
#[test_log::test(tokio::test)]
async fn chat_is_disabled() {
    let mut room = TestRoom::builder().register_module::<ChatModule>().spawn();

    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<ChatModule>(ChatCommand::DisableChat, None)
        .await
        .unwrap();

    assert_eq!(
        alice.receive_event::<ChatModule>().await.unwrap().content,
        ChatEvent::ChatDisabled(ChatDisabled {
            issued_by: alice.id()
        })
    );

    assert_eq!(
        bob.receive_event::<ChatModule>().await.unwrap().content,
        ChatEvent::ChatDisabled(ChatDisabled {
            issued_by: alice.id()
        })
    );

    // Alice cannot send a global message
    alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage(SendMessage {
                content: "Hi there".to_string(),
                scope: Scope::Global,
            }),
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        alice.receive_event::<ChatModule>().await.unwrap().content,
        ChatEvent::Error(ChatError::ChatDisabled)
    );

    // Alice cannot send a private message
    alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage(SendMessage {
                content: "Hi there".to_string(),
                scope: Scope::Private(bob.id()),
            }),
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        alice.receive_event::<ChatModule>().await.unwrap().content,
        ChatEvent::Error(ChatError::ChatDisabled)
    );

    // Bob cannot send a private message
    bob.send_command::<ChatModule>(
        ChatCommand::SendMessage(SendMessage {
            content: "Hi there".to_string(),
            scope: Scope::Private(alice.id()),
        }),
        None,
    )
    .await
    .unwrap();
    assert_eq!(
        bob.receive_event::<ChatModule>().await.unwrap().content,
        ChatEvent::Error(ChatError::ChatDisabled)
    );
}

/// The chat should work after disabling and enabling it again.
#[test_log::test(tokio::test)]
async fn chat_works_after_enabling() {
    let mut room = TestRoom::builder().register_module::<ChatModule>().spawn();

    let mut alice = room.join_alice_moderator(0).await;

    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Disabling the chat should broadcast the ChatDisabled event
    alice
        .send_command::<ChatModule>(ChatCommand::DisableChat, None)
        .await
        .unwrap();
    assert_eq!(
        alice.receive_event::<ChatModule>().await.unwrap().content,
        ChatEvent::ChatDisabled(ChatDisabled {
            issued_by: alice.id()
        })
    );
    assert_eq!(
        bob.receive_event::<ChatModule>().await.unwrap().content,
        ChatEvent::ChatDisabled(ChatDisabled {
            issued_by: alice.id()
        })
    );

    // Enabling the chat should broadcast the ChatEnabled event
    alice
        .send_command::<ChatModule>(ChatCommand::EnableChat, None)
        .await
        .unwrap();
    assert_eq!(
        alice.receive_event::<ChatModule>().await.unwrap().content,
        ChatEvent::ChatEnabled(ChatEnabled {
            issued_by: alice.id()
        })
    );
    assert_eq!(
        bob.receive_event::<ChatModule>().await.unwrap().content,
        ChatEvent::ChatEnabled(ChatEnabled {
            issued_by: alice.id()
        })
    );

    // Alice can send a global message, bob and alice should receive the message
    alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage(SendMessage {
                content: "Hi there".to_string(),
                scope: Scope::Global,
            }),
            None,
        )
        .await
        .unwrap();
    assert_message_eq!(
        &Scope::Global,
        "Hi there",
        alice.id(),
        &alice.receive_event::<ChatModule>().await.unwrap().content,
    );
    assert_message_eq!(
        &Scope::Global,
        "Hi there",
        alice.id(),
        &bob.receive_event::<ChatModule>().await.unwrap().content,
    );

    // Alice can send a private message, bob and alice should receive the message
    alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage(SendMessage {
                content: "Hi there".to_string(),
                scope: Scope::Private(bob.id()),
            }),
            None,
        )
        .await
        .unwrap();
    assert_message_eq!(
        &Scope::Private(bob.id()),
        "Hi there",
        alice.id(),
        &alice.receive_event::<ChatModule>().await.unwrap().content,
    );
    assert_message_eq!(
        &Scope::Private(alice.id()),
        "Hi there",
        alice.id(),
        &bob.receive_event::<ChatModule>().await.unwrap().content,
    );
}

/// Private messages should not be received by participants that are not invited.
#[test_log::test(tokio::test)]
async fn private_messages_are_private() {
    let mut room = TestRoom::builder().register_module::<ChatModule>().spawn();

    let mut alice = room.join_alice_moderator(0).await;

    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let mut charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    // Private messages should not be received by third parties
    alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage(SendMessage {
                content: "Hi there".to_string(),
                scope: Scope::Private(bob.id()),
            }),
            None,
        )
        .await
        .unwrap();
    assert_message_eq!(
        &Scope::Private(bob.id()),
        "Hi there",
        alice.id(),
        &alice.receive_event::<ChatModule>().await.unwrap().content,
    );
    assert_message_eq!(
        &Scope::Private(alice.id()),
        "Hi there",
        alice.id(),
        &bob.receive_event::<ChatModule>().await.unwrap().content,
    );
    assert!(charlie.received_nothing());
}

/// The global chat should be cleared, leaving private messages untouched.
///
/// 1. Alice and Bob join the room
/// 2. Alice sends a global message
/// 3. Alice sends a private message
/// 4. Alice clears the chat
/// 5. Bob leaves and rejoins, the JoinSuccess should only contain the private message
#[test_log::test(tokio::test)]
async fn global_chat_is_cleared() {
    let mut room = TestRoom::builder().register_module::<ChatModule>().spawn();

    // Alice joins
    let mut alice = room.join_alice_moderator(0).await;

    // Bob joins
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // populate the chat history both private and global

    // Alice can send a global message, bob and alice should receive the message
    let global_message = "Hi there";
    alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage(SendMessage {
                content: global_message.to_string(),
                scope: Scope::Global,
            }),
            None,
        )
        .await
        .unwrap();
    assert_message_eq!(
        &Scope::Global,
        global_message,
        alice.id(),
        &alice.receive_event::<ChatModule>().await.unwrap().content,
    );
    assert_message_eq!(
        &Scope::Global,
        global_message,
        alice.id(),
        &bob.receive_event::<ChatModule>().await.unwrap().content,
    );

    // Alice can send a private message, bob and alice should receive the message
    let private_message = "Hi there from alice";
    alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage(SendMessage {
                content: private_message.to_string(),
                scope: Scope::Private(bob.id()),
            }),
            None,
        )
        .await
        .unwrap();
    assert_message_eq!(
        &Scope::Private(bob.id()),
        private_message,
        alice.id(),
        &alice.receive_event::<ChatModule>().await.unwrap().content,
    );
    assert_message_eq!(
        &Scope::Private(alice.id()),
        private_message,
        alice.id(),
        &bob.receive_event::<ChatModule>().await.unwrap().content,
    );

    // clear the chat
    alice
        .send_command::<ChatModule>(ChatCommand::ClearHistory, None)
        .await
        .unwrap();
    assert_eq!(
        alice.receive_event::<ChatModule>().await.unwrap().content,
        ChatEvent::HistoryCleared(HistoryCleared {
            issued_by: alice.id()
        })
    );
    assert_eq!(
        bob.receive_event::<ChatModule>().await.unwrap().content,
        ChatEvent::HistoryCleared(HistoryCleared {
            issued_by: alice.id()
        })
    );

    // When bob reconnects, the join success should only contain the private message
    bob.disconnect();
    let bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let chat_state = bob
        .join_success()
        .get_module::<<ChatModule as SignalingModule>::JoinInfo>()
        .expect("ChatState must be valid")
        .expect("ChatState must exist");

    assert!(chat_state.global_history.is_empty());
    assert_eq!(
        chat_state
            .private_history
            .iter()
            .map(|h| h.correspondent)
            .collect::<Vec<_>>(),
        vec![alice.id()],
        "There should be one private chat with alice"
    );
    assert_eq!(
        chat_state.private_history[0]
            .history
            .iter()
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>(),
        vec![private_message],
        "There should be one message from alice"
    );
}

/// Set the last seen timestamp
///
/// 1. Alice join the room
/// 2. Alice sets last seen timestamp for global and private chat
/// 3. Alice rejoins, the JoinSuccess should contain the last seen timestamps
#[test_log::test(tokio::test)]
async fn last_seen_timestamp_should_be_stored() {
    let timestamp = Timestamp::now();
    let other_participant = ParticipantId::generate();

    let mut room = TestRoom::builder().register_module::<ChatModule>().spawn();

    // Alice joins
    let mut alice = room.join_alice_moderator(0).await;

    // set global last seen timestamp
    alice
        .send_command::<ChatModule>(
            ChatCommand::SetLastSeenTimestamp(SetLastSeenTimestamp {
                scope: Scope::Global,
                timestamp,
            }),
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        alice.receive_event::<ChatModule>().await.unwrap().content,
        ChatEvent::SetLastSeenTimestamp(SetLastSeenTimestamp {
            scope: Scope::Global,
            timestamp,
        })
    );

    // set private last seen timestamp
    alice
        .send_command::<ChatModule>(
            ChatCommand::SetLastSeenTimestamp(SetLastSeenTimestamp {
                scope: Scope::Private(other_participant),
                timestamp,
            }),
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        alice.receive_event::<ChatModule>().await.unwrap().content,
        ChatEvent::SetLastSeenTimestamp(SetLastSeenTimestamp {
            scope: Scope::Private(other_participant),
            timestamp,
        })
    );

    alice.disconnect();
    let alice = room.join_alice_moderator(0).await;
    let chat_state = alice
        .join_success()
        .get_module::<<ChatModule as SignalingModule>::JoinInfo>()
        .expect("ChatState must be valid")
        .expect("ChatState must exist");
    assert_eq!(chat_state.last_seen_timestamp_global, Some(timestamp));
    assert_eq!(
        chat_state
            .last_seen_timestamps_private
            .get(&other_participant)
            .copied(),
        Some(timestamp)
    );
}

/// Send a message in the breakout room scope
///
/// Uses the scenario from [start_breakout_scenario]
/// - Alice & Bob are in breakout room 1
/// - Charlie is in the main room
///
/// 1. Alice sends a message with the breakout room 1 scope
/// 2. Alice and Bob receive the message
/// 3. Charlie won't receive anything
#[test_log::test(tokio::test)]
async fn breakout_scope_messages() {
    let breakout_scenario = start_breakout_scenario().await;

    let mut breakout_alice = breakout_scenario.alice;
    let mut breakout_bob = breakout_scenario.bob;
    let mut main_room_charlie = breakout_scenario.charlie;

    let breakout_message: String = "breakout_message1".into();
    let breakout_scope = Scope::Breakout(BreakoutId::from(1));

    // Alice sends a message to the breakout room
    breakout_alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage(SendMessage {
                content: breakout_message.clone(),
                scope: breakout_scope.clone(),
            }),
            None,
        )
        .await
        .unwrap();

    // Alice receives the message
    assert_message_eq!(
        &breakout_scope,
        breakout_message,
        breakout_alice.id(),
        &breakout_alice
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .content,
    );

    // Bob receives the message
    assert_message_eq!(
        &breakout_scope,
        breakout_message,
        breakout_alice.id(),
        &breakout_bob
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .content,
    );

    // Charlie is clueless
    assert!(main_room_charlie.received_nothing());
}

/// Send a message in the breakout room scope
///
/// Uses the scenario from [start_breakout_scenario]
/// - Alice & Bob are in breakout room 1
/// - Charlie is in the main room
///
/// 1. Charlie attempts to send a message to breakout room 1
/// 2. Charlie receives an error
#[test_log::test(tokio::test)]
async fn invalid_breakout_scope() {
    let breakout_scenario = start_breakout_scenario().await;

    let mut breakout_alice = breakout_scenario.alice;
    let mut breakout_bob = breakout_scenario.bob;
    let mut main_room_charlie = breakout_scenario.charlie;

    let breakout_message: String = "breakout_message1".into();
    let breakout_scope = Scope::Breakout(BreakoutId::from(1));

    // Charlie sends a message to breakout room 1
    main_room_charlie
        .send_command::<ChatModule>(
            ChatCommand::SendMessage(SendMessage {
                content: breakout_message.clone(),
                scope: breakout_scope.clone(),
            }),
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        main_room_charlie
            .receive::<ChatEvent>()
            .await
            .unwrap()
            .content,
        ChatEvent::Error(ChatError::InvalidBreakoutScope)
    );

    assert!(breakout_alice.received_nothing());
    assert!(breakout_bob.received_nothing());
}

/// Send a message in global room scope from a breakout room
///
/// Uses the scenario from [start_breakout_scenario]
/// - Alice & Bob are in breakout room 1
/// - Charlie is in the main room
///
/// 1. Alice sends a message with global scope
/// 2. Alice, Bob and Charlie receive the message
#[test_log::test(tokio::test)]
async fn send_global_message_from_breakout_room() {
    let breakout_scenario = start_breakout_scenario().await;

    let mut breakout_alice = breakout_scenario.alice;
    let mut breakout_bob = breakout_scenario.bob;
    let mut main_room_charlie = breakout_scenario.charlie;

    let global_message_from_breakout_room: String = "global message from breakout room".into();
    let global_scope = Scope::Global;

    // Alice sends a message to the global room
    breakout_alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage(SendMessage {
                content: global_message_from_breakout_room.clone(),
                scope: global_scope.clone(),
            }),
            None,
        )
        .await
        .unwrap();

    assert_message_eq!(
        &global_scope,
        global_message_from_breakout_room,
        breakout_alice.id(),
        &breakout_alice
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .content,
    );

    assert_message_eq!(
        &global_scope,
        global_message_from_breakout_room,
        breakout_alice.id(),
        &breakout_bob
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .content,
    );

    assert_message_eq!(
        &global_scope,
        global_message_from_breakout_room,
        breakout_alice.id(),
        &main_room_charlie
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .content,
    );
}

/// Send a message in global room scope from the main room to a breakout room
///
/// Uses the scenario from [start_breakout_scenario]
/// - Alice & Bob are in breakout room 1
/// - Charlie is in the main room
///
/// 3. Charlie sends a message with global scope
/// 4. Alice, Bob and Charlie receive the message
#[test_log::test(tokio::test)]
async fn send_global_message_to_breakout_room() {
    let breakout_scenario = start_breakout_scenario().await;

    let mut breakout_alice = breakout_scenario.alice;
    let mut breakout_bob = breakout_scenario.bob;
    let mut main_room_charlie = breakout_scenario.charlie;

    let global_message_from_breakout_room: String = "global message from breakout room".into();
    let global_scope = Scope::Global;

    // Charlie sends a message to the global room
    main_room_charlie
        .send_command::<ChatModule>(
            ChatCommand::SendMessage(SendMessage {
                content: global_message_from_breakout_room.clone(),
                scope: global_scope.clone(),
            }),
            None,
        )
        .await
        .unwrap();

    assert_message_eq!(
        &global_scope,
        global_message_from_breakout_room,
        main_room_charlie.id(),
        &breakout_alice
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .content,
    );

    assert_message_eq!(
        &global_scope,
        global_message_from_breakout_room,
        main_room_charlie.id(),
        &breakout_bob
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .content,
    );

    assert_message_eq!(
        &global_scope,
        global_message_from_breakout_room,
        main_room_charlie.id(),
        &main_room_charlie
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .content,
    );
}

/// Send a message with private scope between breakout rooms
///
/// Uses the scenario from [start_breakout_scenario]
/// - Alice & Bob are in breakout room 1
/// - Charlie is in the main room
///
/// 1. Alice sends a private message to Bob (Breakout1 -> Breakout1)
/// 2. Alice and Bob receive the message
#[test_log::test(tokio::test)]
async fn send_private_message_breakout_to_breakout() {
    let breakout_scenario = start_breakout_scenario().await;

    let mut breakout_alice = breakout_scenario.alice;
    let mut breakout_bob = breakout_scenario.bob;
    let mut main_room_charlie = breakout_scenario.charlie;

    // Alice sends a private message to Bob (Breakout1 -> Breakout1)
    let message: String = "private message from breakout to breakout room".into();
    breakout_alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage(SendMessage {
                content: message.clone(),
                scope: Scope::Private(breakout_bob.id()),
            }),
            None,
        )
        .await
        .unwrap();

    // Alice receives her own message
    assert_message_eq!(
        &Scope::Private(breakout_bob.id()),
        message,
        breakout_alice.id(),
        &breakout_alice
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .content,
    );

    // Bob receives the message from Alice
    assert_message_eq!(
        &Scope::Private(breakout_alice.id()),
        message,
        breakout_alice.id(),
        &breakout_bob
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .content,
    );

    // Charlie wasn't involved
    assert!(main_room_charlie.received_nothing());
}

/// Send a message with private scope between a breakout room and the main room
///
/// Uses the scenario from [start_breakout_scenario]
/// - Alice & Bob are in breakout room 1
/// - Charlie is in the main room
///
/// 1. Bob sends a private message to Charlie (Breakout1 -> Main)
/// 2. Bob and Charlie receive the message
#[test_log::test(tokio::test)]
async fn send_private_message_breakout_to_main() {
    let breakout_scenario = start_breakout_scenario().await;

    let mut breakout_alice = breakout_scenario.alice;
    let mut breakout_bob = breakout_scenario.bob;
    let mut main_room_charlie = breakout_scenario.charlie;

    // Bob sends a private message to Charlie (Breakout1 -> Main)
    let message: String = "private message from breakout to main room".into();
    breakout_bob
        .send_command::<ChatModule>(
            ChatCommand::SendMessage(SendMessage {
                content: message.clone(),
                scope: Scope::Private(main_room_charlie.id()),
            }),
            None,
        )
        .await
        .unwrap();

    // Bob receives his own message
    assert_message_eq!(
        &Scope::Private(main_room_charlie.id()),
        message,
        breakout_bob.id(),
        &breakout_bob
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .content,
    );

    // Charlie receives the message from Bob
    assert_message_eq!(
        &Scope::Private(breakout_bob.id()),
        message,
        breakout_bob.id(),
        &main_room_charlie
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .content,
    );

    // Alice wasn't involved
    assert!(breakout_alice.received_nothing());
}

/// Send a message with private scope between the main and a breakout room
///
/// Uses the scenario from [start_breakout_scenario]
/// - Alice & Bob are in breakout room 1
/// - Charlie is in the main room
///
/// 1. Charlie sends a private message to Alice (Main -> Breakout1)
/// 2. Charlie and Alice receive the message
#[test_log::test(tokio::test)]
async fn send_private_message_main_to_breakout() {
    let breakout_scenario = start_breakout_scenario().await;

    let mut breakout_alice = breakout_scenario.alice;
    let mut breakout_bob = breakout_scenario.bob;
    let mut main_room_charlie = breakout_scenario.charlie;

    // Charlie sends a private message to Alice (Main -> Breakout1)
    let message: String = "private message from main to breakout room".into();
    main_room_charlie
        .send_command::<ChatModule>(
            ChatCommand::SendMessage(SendMessage {
                content: message.clone(),
                scope: Scope::Private(breakout_alice.id()),
            }),
            None,
        )
        .await
        .unwrap();

    // Charlie receives his own message
    assert_message_eq!(
        &Scope::Private(breakout_alice.id()),
        message,
        main_room_charlie.id(),
        &main_room_charlie
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .content,
    );

    // Alice receives the message from charlie
    assert_message_eq!(
        &Scope::Private(main_room_charlie.id()),
        message,
        main_room_charlie.id(),
        &breakout_alice
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .content,
    );

    // Bob wasn't involved
    assert!(breakout_bob.received_nothing());
}

/// Return type for [start_breakout_scenario]
struct BreakoutScenario<S> {
    _room: TestRoom,
    alice: MockParticipant<S>,
    bob: MockParticipant<S>,
    charlie: MockParticipant<S>,
}

/// Starts three breakout rooms with alice, bob and charlie
///
/// Alice and Bob will join the first breakout room but charlie will stay in the main room
async fn start_breakout_scenario() -> BreakoutScenario<JoinSuccess> {
    let mut room = TestRoom::builder().register_module::<ChatModule>().spawn();

    let mut alice = room.join_alice_moderator(1).await;
    let mut bob = room.join_bob(1).await;
    let mut charlie = room.join_charlie(1).await;

    // bob joined for alice
    assert!(matches!(
        alice.receive::<CoreEvent>().await.unwrap().content,
        CoreEvent::ParticipantConnected { .. }
    ));

    // charlie joined for alice
    assert!(matches!(
        alice.receive::<CoreEvent>().await.unwrap().content,
        CoreEvent::ParticipantConnected { .. }
    ));

    // charlie joined for bob
    assert!(matches!(
        bob.receive::<CoreEvent>().await.unwrap().content,
        CoreEvent::ParticipantConnected { .. }
    ));

    // alice starts breakout rooms
    alice
        .send_breakout_command(
            BreakoutCommand::Start(BreakoutConfig {
                rooms: vec![
                    BreakoutRoomConfig {
                        name: "Room 0".into(),
                        assignments: vec![],
                    },
                    BreakoutRoomConfig {
                        name: "Room 1".into(),
                        assignments: vec![],
                    },
                    BreakoutRoomConfig {
                        name: "Room 2".into(),
                        assignments: vec![],
                    },
                ],
                duration: None,
            }),
            None,
        )
        .await
        .unwrap();

    // breakout rooms started
    let _: BreakoutEvent = alice.receive().await.unwrap().content;
    let _: BreakoutEvent = bob.receive().await.unwrap().content;
    let _: BreakoutEvent = charlie.receive().await.unwrap().content;

    // Alice switches to breakout room 1
    alice
        .send_breakout_command(
            BreakoutCommand::SwitchRoom(RoomKind::Breakout(BreakoutId::from(1))),
            None,
        )
        .await
        .unwrap();
    let _: BreakoutEvent = alice.receive().await.unwrap().content;
    let _: BreakoutEvent = bob.receive().await.unwrap().content;
    let _: BreakoutEvent = charlie.receive().await.unwrap().content;

    // Bob switches to breakout room 1
    bob.send_breakout_command(
        BreakoutCommand::SwitchRoom(RoomKind::Breakout(BreakoutId::from(1))),
        None,
    )
    .await
    .unwrap();
    let _: BreakoutEvent = alice.receive().await.unwrap().content;
    let _: BreakoutEvent = bob.receive().await.unwrap().content;
    let _: BreakoutEvent = charlie.receive().await.unwrap().content;

    assert!(alice.received_nothing());
    assert!(bob.received_nothing());
    assert!(charlie.received_nothing());

    BreakoutScenario {
        _room: room,
        alice,
        bob,
        charlie,
    }
}
