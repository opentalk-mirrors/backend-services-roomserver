// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{collections::BTreeMap, path::Path, sync::Arc};

use chrono_tz::Tz;
use opentalk_roomserver_report_generation::ToReportDateTime;
use opentalk_roomserver_room::{AssetUploaded, StorageProvider};
use opentalk_roomserver_signaling::{
    module_context::ModuleContext,
    signaling_module::{JoinInfo, SignalingModule, SignalingModuleInitData},
    storage::AssetMetaData,
};
use opentalk_roomserver_types::{
    connection_id::ConnectionId, room_parameters::EventContext,
    signaling::module_error::SignalingModuleError,
};
use opentalk_roomserver_types_meeting_report::{
    MEETING_REPORT_MODULE_ID,
    command::MeetingReportCommand,
    event::{MeetingReportError, MeetingReportEvent, PdfAsset},
};
use opentalk_types_common::{
    assets::{AssetFileKind, FileExtension, asset_file_kind},
    events::{EventDescription, EventTitle},
    modules::ModuleId,
    time::{TimeZone, Timestamp},
};
use opentalk_types_signaling::ParticipantId;

use crate::template::{ReportParticipant, ReportTemplateParameter};

mod template;

pub struct MeetingReportModule;

const DEFAULT_TEMPLATE: &str = include_str!("attendance_report.typ");

impl SignalingModule for MeetingReportModule {
    const NAMESPACE: ModuleId = MEETING_REPORT_MODULE_ID;

    type Incoming = MeetingReportCommand;

    type Outgoing = MeetingReportEvent;

    type Loopback = Result<AssetUploaded, SignalingModuleError<MeetingReportError>>;

    type JoinInfo = ();

    type PeerJoinInfo = ();

    type Error = MeetingReportError;

    fn init(_init_data: SignalingModuleInitData) -> Option<Self> {
        Some(Self)
    }

    #[allow(unused_variables)]
    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        is_first_connection: bool,
    ) -> Result<JoinInfo<Self>, SignalingModuleError<Self::Error>> {
        Ok(JoinInfo::default())
    }

    #[allow(unused_variables)]
    fn on_participant_disconnected(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        Ok(())
    }

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        _connection_id: ConnectionId,
        content: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match content {
            MeetingReportCommand::GenerateAttendanceReport {
                include_email_addresses,
            } => self.generate_attendance_report(ctx, sender, include_email_addresses)?,
        };
        Ok(())
    }

    fn on_loopback_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: Self::Loopback,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match event {
            Ok(uploaded) => {
                let pdf_asset = PdfAsset {
                    filename: uploaded.filename,
                    asset_id: uploaded.id,
                };
                tracing::debug!("Generated meeting attendance report: {pdf_asset:?}");
                ctx.send_ws_message(
                    ctx.participants.connected().ids(),
                    MeetingReportEvent::PdfAsset(pdf_asset),
                )?;
                Ok(())
            }
            Err(err) => {
                tracing::error!("Failed to upload meeting report: {err:?}");
                Err(err)
            }
        }
    }
}

impl MeetingReportModule {
    fn generate_attendance_report(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        include_email_addresses: bool,
    ) -> Result<(), SignalingModuleError<MeetingReportError>> {
        if !ctx.is_moderator(sender) {
            return Err(MeetingReportError::InsufficientPermissions.into());
        }

        let storage = ctx.storage();
        let event = ctx.room_task_info().room.event.as_ref();

        let report_timezone = event
            .and_then(|event| event.timezone)
            .unwrap_or(TimeZone::from(Tz::UTC));
        let tz = Tz::from(report_timezone);

        let participants = ctx
            .participants
            .visible()
            .iter()
            .map(|(id, state)| ReportParticipant {
                id: *id,
                name: state.display_name.to_string(),
                role: (state.role, state.kind).into(),
                kind: state.kind,
                email: include_email_addresses
                    .then(|| state.email.clone())
                    .flatten()
                    .unwrap_or_default(),
                joined_at: state.joined_at.to_report_date_time(&tz),
                left_at: state.left_at.to_report_date_time(&tz),
            })
            .collect();

        let event = ctx.room_task_info().room.event.clone();

        ctx.spawn(async move {
            Self::generate_report(storage, report_timezone, tz, participants, event).await
        });

        Ok(())
    }

    fn generate_pdf_report_from_template(
        template: String,
        parameter: &ReportTemplateParameter,
    ) -> Result<Vec<u8>, SignalingModuleError<MeetingReportError>> {
        let serialized = serde_json::to_string(parameter)
            .map_err(|_| MeetingReportError::Generate)?
            .into_bytes()
            .into();
        let report = opentalk_roomserver_report_generation::generate_pdf_report(
            template,
            BTreeMap::from_iter([(Path::new("data.json"), serialized)]),
        )
        .map_err(|_| MeetingReportError::Generate)?;

        Ok(report)
    }

    async fn generate_report(
        storage: Arc<dyn StorageProvider>,
        report_timezone: TimeZone,
        tz: Tz,
        participants: Vec<ReportParticipant>,
        event: Option<EventContext>,
    ) -> Result<AssetUploaded, SignalingModuleError<MeetingReportError>> {
        let quota = storage.remaining_quota().await;
        if quota == 0 {
            return Err(MeetingReportError::StorageExceeded.into());
        }

        let event = event.as_ref();
        let starts_at = event
            .and_then(|event| event.starts_at)
            .to_report_date_time(&tz);
        let ends_at = event
            .and_then(|event| event.ends_at)
            .to_report_date_time(&tz);
        let title = event
            .map(|event| event.title.clone())
            .unwrap_or_else(|| EventTitle::from_str_lossy(""));
        let description = event
            .map(|event| event.description.clone())
            .unwrap_or_else(|| EventDescription::from_str_lossy(""));
        let report = Self::generate_pdf_report_from_template(
            DEFAULT_TEMPLATE.to_string(),
            &ReportTemplateParameter {
                title,
                description,
                starts_at,
                ends_at,
                report_timezone,
                participants,
            },
        )?;

        const ASSET_FILE_KIND: AssetFileKind = asset_file_kind!("meeting_report");
        let upload = storage
            .upload_file(
                report,
                AssetMetaData {
                    kind: ASSET_FILE_KIND,
                    timestamp: Timestamp::now(),
                    extension: FileExtension::pdf(),
                },
            )
            .await
            .map_err(MeetingReportError::from)?;
        Ok(upload)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::{DEFAULT_TEMPLATE, MeetingReportModule, template::ReportTemplateParameter};

    fn generate(parameter: &ReportTemplateParameter) -> String {
        let pdf = MeetingReportModule::generate_pdf_report_from_template(
            DEFAULT_TEMPLATE.to_string(),
            parameter,
        )
        .expect("generation should work");
        pdf_extract::extract_text_from_mem(&pdf)
            .expect("text should be extractable from generated pdf")
    }

    #[test]
    fn generate_report_small() {
        assert_snapshot!(generate(&crate::template::tests::example_small()), @r"
        Attendance Report
         Meeting : Testmeeting

        Report timezone : Europe/Berlin

        Participants
         Nr Name Role

        1 Alice Adams Moderator
        ");
    }

    #[test]
    fn generate_report_medium() {
        assert_snapshot!(
            generate(
                &crate::template::tests::example_medium()
            ),
            @r"
        Attendance Report
         Meeting : Testmeeting

        Details : A medium sized test meeting

        Start : 2025-02-06 08:18

        End : 2025-02-06 11:25

        Report timezone : Europe/Berlin

        Participants
         Nr Name Role

        1 Alice Adams Moderator

        2 Charlie Cooper User

        3 Bob Burton User
        "
        );
    }

    #[test]
    fn generate_report_large() {
        assert_snapshot!(generate(&crate::template::tests::example_large()), @r"
        Attendance Report
         Meeting : Large Testmeeting

        Details : The large test meeting

        Start : 2025-02-06 08:18

        End : 2025-02-06 11:25

        Report timezone : Europe/Berlin

        Participants
         Nr Name Role

        1 Alice Adams Moderator

        2 Franz Fischer User

        3 Recorder User

        4 Charlie Cooper User

        5 Bob Burton User

        6 Erin Guest

        7 Dave Dunn Guest
        ");
    }
}
