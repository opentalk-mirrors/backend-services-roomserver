// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use insta::assert_snapshot;
use opentalk_roomserver_module_meeting_report::MeetingReportModule;
use opentalk_roomserver_room::mocking::room::{TestRoom, flush_connected_events};
use opentalk_roomserver_types_meeting_report::{
    command::MeetingReportCommand,
    event::{MeetingReportError, MeetingReportEvent, PdfAsset},
};

#[test_log::test(tokio::test)]
async fn generate_meeting_report() {
    let mut room = TestRoom::builder()
        .register_module::<MeetingReportModule>()
        .spawn();

    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let _gustav = room.join_gustav_guest().await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    alice
        .send_command::<MeetingReportModule>(
            MeetingReportCommand::GenerateAttendanceReport {
                include_email_addresses: true,
            },
            None,
        )
        .await
        .unwrap();

    assert!(matches!(
        alice
            .receive_event::<MeetingReportModule>()
            .await
            .unwrap()
            .content,
        MeetingReportEvent::PdfAsset(PdfAsset { .. })
    ));

    let path = &room.stored_files()[0];
    // Title, details, start & end are missing because they do not exist in the
    // TestRoom
    assert_snapshot!(pdf_extract::extract_text(path).unwrap(), @r"
    Attendance Report
     Meeting :

    Report timezone : Europe/Berlin

    Participants
     Nr Name Role

    1 Alice the angry Moderator

    2 Bob the bold User

    3 Gustav the great Guest
    ");
}

#[test_log::test(tokio::test)]
async fn quota_exceeded() {
    let mut room = TestRoom::builder()
        .storage_quota(0)
        .register_module::<MeetingReportModule>()
        .spawn();

    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<MeetingReportModule>(
            MeetingReportCommand::GenerateAttendanceReport {
                include_email_addresses: false,
            },
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        alice
            .receive_event::<MeetingReportModule>()
            .await
            .unwrap()
            .content,
        MeetingReportEvent::Error(MeetingReportError::StorageExceeded)
    );
}

#[test_log::test(tokio::test)]
async fn insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<MeetingReportModule>()
        .spawn();
    let mut bob = room.join_bob(0).await;

    bob.send_command::<MeetingReportModule>(
        MeetingReportCommand::GenerateAttendanceReport {
            include_email_addresses: true,
        },
        None,
    )
    .await
    .unwrap();

    assert_eq!(
        bob.receive_event::<MeetingReportModule>()
            .await
            .unwrap()
            .content,
        MeetingReportEvent::Error(MeetingReportError::InsufficientPermissions)
    );
}
