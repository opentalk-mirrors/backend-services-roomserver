// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_module_chat::ChatModule;
use opentalk_roomserver_module_moderation::ModerationModule;
use opentalk_roomserver_room::mocking::room::TestRoom;
use opentalk_roomserver_types::{
    breakout::{
        breakout_config::{BreakoutConfig, BreakoutRoomConfig},
        command::BreakoutCommand,
        event::BreakoutEvent,
    },
    core::{CoreCommand, CoreEvent},
};
use opentalk_roomserver_types_chat::{
    Scope,
    command::{ChatCommand, SendMessage},
    event::ChatEvent,
};
use opentalk_roomserver_types_moderation::{
    command::{Accept, ModerationCommand},
    event::ModerationEvent,
};

#[test_log::test(tokio::test)]
async fn waiting_participants_dont_receive_messages() {
    let mut room = TestRoom::builder()
        .register_module::<ChatModule>()
        .register_module::<ModerationModule>()
        .waiting_room(true)
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.waiting_room_bob(0).await;

    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert!(matches!(event.payload, CoreEvent::JoinedWaitingRoom { .. }));

    // Chat message is used as an arbitrary command that should not be sent to
    // waiting participants
    alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage(SendMessage {
                content: "Bob can not read this".into(),
                scope: Scope::Global,
            }),
            None,
        )
        .await
        .unwrap();
    let event = alice.receive_event::<ChatModule>().await.unwrap();
    assert!(matches!(event.payload, ChatEvent::MessageSent(..)));

    // Bob should not receive the event
    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
async fn waiting_participants_dont_receive_broadcasts() {
    let mut room = TestRoom::builder()
        .register_module::<ChatModule>()
        .register_module::<ModerationModule>()
        .waiting_room(true)
        .spawn();

    let mut alice = room.join_alice_moderator(0).await;

    let mut bob = room.waiting_room_bob(0).await;
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert!(matches!(event.payload, CoreEvent::JoinedWaitingRoom { .. }));

    let mut charlie = room.waiting_room_charlie(0).await;
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert!(matches!(event.payload, CoreEvent::JoinedWaitingRoom { .. }));

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::Accept(Accept {
                target: charlie.id(),
            }),
            None,
        )
        .await
        .unwrap();

    let event = charlie
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, ModerationEvent::Accepted);

    charlie
        .send_core_command(CoreCommand::EnterRoom, None)
        .await
        .unwrap();
    let mut charlie = charlie.join_success().await.unwrap();

    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert!(matches!(event.payload, CoreEvent::LeftWaitingRoom { .. }));

    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert!(matches!(
        event.payload,
        CoreEvent::ParticipantConnected { .. }
    ));

    assert!(bob.received_nothing());

    // Alice sends a command that triggers a broadcast
    alice
        .send_breakout_command(
            BreakoutCommand::Start(BreakoutConfig {
                rooms: Vec::from_iter([BreakoutRoomConfig {
                    name: "Room 1".into(),
                    assignments: Vec::new(),
                }]),
                duration: None,
            }),
            None,
        )
        .await
        .unwrap();

    let event = alice.receive::<BreakoutEvent>().await.unwrap();
    assert!(matches!(event.payload, BreakoutEvent::Started { .. }));

    let event = charlie.receive::<BreakoutEvent>().await.unwrap();
    assert!(matches!(event.payload, BreakoutEvent::Started { .. }));

    assert!(bob.received_nothing());

    charlie.disconnect().await.unwrap();

    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert!(matches!(
        event.payload,
        CoreEvent::ParticipantDisconnected { .. }
    ));

    assert!(bob.received_nothing());
}
