// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_module_livekit::LiveKitModule;
use opentalk_roomserver_room::mocking::room::flush_connected_events;
use opentalk_roomserver_types_livekit::{Credentials, LiveKitCommand, LiveKitEvent};
use pretty_assertions::assert_eq;

mod common;

#[test_log::test(tokio::test)]
#[ignore]
async fn request_access_token() {
    let (_container, mut room, public_url) = common::build_livekit_room().await;

    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<LiveKitModule>(LiveKitCommand::CreateNewAccessToken, None)
        .await
        .unwrap();
    let token_event = alice.receive_event::<LiveKitModule>().await.unwrap();
    assert!(bob.received_nothing());

    assert!(matches!(
        token_event.content,
        LiveKitEvent::Credentials(Credentials { .. })
    ));

    let LiveKitEvent::Credentials(credential) = token_event.content else {
        unreachable!()
    };

    assert_eq!(credential.room, room.id().to_string());
    assert_eq!(credential.public_url, public_url);
    assert_eq!(credential.service_url, None);
    assert!(!credential.token.is_empty());
}
