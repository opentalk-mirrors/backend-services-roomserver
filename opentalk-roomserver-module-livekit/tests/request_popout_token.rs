// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_module_livekit::{
    LiveKitModule, command::LiveKitCommand, event::LiveKitEvent,
};
use opentalk_roomserver_types::core_event::CoreEvent;

mod common;

#[test_log::test(tokio::test)]
#[ignore]
async fn request_access_token() {
    let (_container, mut room, _public_url) = common::build_livekit_room().await;

    let mut alice = room.join_alice_moderator().await;

    let mut bob = room.join_bob().await;
    assert!(matches!(
        alice.receive::<CoreEvent>().await.unwrap().content,
        CoreEvent::ParticipantConnected { .. }
    ));

    alice
        .send_command::<LiveKitModule>(LiveKitCommand::RequestPopoutStreamAccessToken, None)
        .await
        .unwrap();
    let token_event = alice.receive_event::<LiveKitModule>().await.unwrap();
    assert!(bob.received_nothing());

    assert!(matches!(
        token_event.content,
        LiveKitEvent::PopoutStreamAccessToken { .. }
    ));
}
