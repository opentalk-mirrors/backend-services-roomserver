// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `meeting_report` namespace

use serde::{Deserialize, Serialize};

/// Incoming websocket messages
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum MeetingReportCommand {
    /// Generate a report on current and past meeting attendees
    GenerateAttendanceReport {
        /// Wether or not to include the e-mail addresses of the participants in the
        /// report
        include_email_addresses: bool,
    },
}

#[cfg(test)]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::MeetingReportCommand;

    #[test]
    fn create_attendees_report() {
        let json = json!({
            "action": "generate_attendance_report",
            "include_email_addresses": false,
        });

        assert_eq!(
            serde_json::from_value::<MeetingReportCommand>(json).unwrap(),
            MeetingReportCommand::GenerateAttendanceReport {
                include_email_addresses: false
            }
        );
    }
}
