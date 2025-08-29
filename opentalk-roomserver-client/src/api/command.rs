// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_types::{
    breakout::{BREAKOUT_MODULE_ID, command::BreakoutCommand},
    core::{CORE_MODULE_ID, CoreCommand},
};
use opentalk_roomserver_types_automod::{AUTOMOD_MODULE_ID, command::AutomodCommand};
use opentalk_roomserver_types_chat::{CHAT_MODULE_ID, command::ChatCommand};
use opentalk_roomserver_types_echo::{ECHO_MODULE_ID, command::EchoCommand};
use opentalk_roomserver_types_meeting_report::{
    MEETING_REPORT_MODULE_ID, command::MeetingReportCommand,
};
use opentalk_roomserver_types_moderation::{MODERATION_MODULE_ID, command::ModerationCommand};
use opentalk_roomserver_types_polls::{POLLS_MODULE_ID, command::PollsCommand};
use opentalk_roomserver_types_raise_hands::{RAISE_HANDS_MODULE_ID, command::RaiseHandsCommand};
use opentalk_roomserver_types_subroom_audio::{
    SUBROOM_AUDIO_MODULE_ID, command::SubroomAudioCommand,
};
use opentalk_roomserver_types_timer::{TIMER_MODULE_ID, TimerCommand};
use opentalk_types_common::modules::ModuleId;
use serde::{Deserialize, Serialize};
// reexport commands for easier usage
pub use {
    opentalk_roomserver_types_e2ee::{E2EE_MODULE_ID, E2eeCommand},
    opentalk_roomserver_types_livekit::{
        LIVEKIT_MODULE_ID, LiveKitCommand, MicrophoneRestrictionState,
    },
    opentalk_roomserver_types_shared_folder::{
        SHARED_FOLDER_MODULE_ID, command::SharedFolderCommand,
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct SignalingCommand {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<u64>,
    #[serde(flatten)]
    pub payload: SignalingModuleCommand,
}

impl SignalingCommand {
    pub fn namespace(&self) -> ModuleId {
        self.payload.namespace()
    }
}

impl From<SignalingModuleCommand> for SignalingCommand {
    fn from(payload: SignalingModuleCommand) -> Self {
        Self {
            transaction_id: None,
            payload,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "namespace", content = "payload", rename_all = "snake_case")]
pub enum SignalingModuleCommand {
    Core(CoreCommand),
    Automod(AutomodCommand),
    Breakout(BreakoutCommand),
    Echo(EchoCommand),
    Chat(ChatCommand),

    #[serde(rename = "livekit")]
    LiveKit(LiveKitCommand),

    E2ee(E2eeCommand),

    Timer(TimerCommand),
    Polls(PollsCommand),
    SharedFolder(SharedFolderCommand),
    MeetingReport(MeetingReportCommand),
    Moderation(ModerationCommand),
    RaiseHands(RaiseHandsCommand),
    SubroomAudio(SubroomAudioCommand),
}

impl SignalingModuleCommand {
    pub fn namespace(&self) -> ModuleId {
        match self {
            Self::Core(..) => CORE_MODULE_ID,
            Self::Automod(..) => AUTOMOD_MODULE_ID,
            Self::Breakout(..) => BREAKOUT_MODULE_ID,
            Self::Echo(..) => ECHO_MODULE_ID,
            Self::Chat(..) => CHAT_MODULE_ID,
            Self::LiveKit(..) => LIVEKIT_MODULE_ID,
            Self::E2ee(..) => E2EE_MODULE_ID,
            Self::Timer(..) => TIMER_MODULE_ID,
            Self::Polls(..) => POLLS_MODULE_ID,
            Self::SharedFolder(..) => SHARED_FOLDER_MODULE_ID,
            Self::MeetingReport(..) => MEETING_REPORT_MODULE_ID,
            Self::Moderation(..) => MODERATION_MODULE_ID,
            Self::RaiseHands(..) => RAISE_HANDS_MODULE_ID,
            Self::SubroomAudio(..) => SUBROOM_AUDIO_MODULE_ID,
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_roomserver_types::{breakout::command::BreakoutCommand, core::CoreCommand};
    use opentalk_roomserver_types_automod::command::AutomodCommand;
    use opentalk_roomserver_types_chat::command::ChatCommand;
    use opentalk_roomserver_types_echo::command::EchoCommand;
    use opentalk_roomserver_types_livekit::LiveKitCommand;
    use opentalk_roomserver_types_meeting_report::command::MeetingReportCommand;
    use opentalk_roomserver_types_moderation::command::{Accept, ModerationCommand};
    use opentalk_roomserver_types_polls::{
        ChoiceId, PollId,
        command::{Choices, PollsCommand, Vote},
    };
    use opentalk_roomserver_types_raise_hands::command::RaiseHandsCommand;
    use opentalk_roomserver_types_timer::{Start, TimerCommand, command::Kind};
    use opentalk_types_common::modules::ModuleId;
    use opentalk_types_signaling::ParticipantId;
    use serde::Deserialize;

    use super::SignalingModuleCommand;
    use crate::api::command::SignalingCommand;

    #[derive(Debug, Clone, Deserialize)]
    pub struct NamespaceOnly {
        pub namespace: ModuleId,
    }

    #[test]
    fn serialize_command_core() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::Core(CoreCommand::EnterRoom),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();
        assert_snapshot!(raw, @r#"
        {
          "namespace": "core",
          "payload": {
            "action": "enter_room"
          }
        }
        "#);
    }

    #[test]
    fn serialize_command_automod() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::Automod(AutomodCommand::Stop),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "automod",
          "payload": {
            "action": "stop"
          }
        }
        "#);
    }

    #[test]
    fn serialize_command_breakout() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::Breakout(BreakoutCommand::Stop { delay: None }),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "breakout",
          "payload": {
            "action": "stop"
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, command.namespace());
    }

    #[test]
    fn serialize_command_chat() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::Chat(ChatCommand::DisableChat),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "chat",
          "payload": {
            "action": "disable_chat"
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, command.namespace());
    }

    #[test]
    fn serialize_command_echo() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::Echo(EchoCommand::Ping),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "echo",
          "payload": {
            "action": "ping"
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, command.namespace());
    }

    #[test]
    fn serialize_command_livekit() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::LiveKit(LiveKitCommand::CreateNewAccessToken),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "livekit",
          "payload": {
            "action": "create_new_access_token"
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, command.namespace());
    }

    #[test]
    fn serialize_command_timer() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::Timer(TimerCommand::Start(Start {
                kind: Kind::Stopwatch,
                style: None,
                title: None,
                enable_ready_check: false,
            })),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "timer",
          "payload": {
            "action": "start",
            "kind": "stopwatch",
            "style": null,
            "title": null,
            "enable_ready_check": false
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, command.namespace());
    }

    #[test]
    fn serialize_command_polls() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::Polls(PollsCommand::Vote(Vote {
                poll_id: PollId::nil(),
                choices: Choices::Single {
                    choice_id: ChoiceId::from_u32(0),
                },
            })),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "polls",
          "payload": {
            "action": "vote",
            "poll_id": "00000000-0000-0000-0000-000000000000",
            "choice_id": 0
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, command.namespace());
    }

    #[test]
    fn serialize_command_meeting_report() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::MeetingReport(
                MeetingReportCommand::GenerateAttendanceReport {
                    include_email_addresses: false,
                },
            ),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "meeting_report",
          "payload": {
            "action": "generate_attendance_report",
            "include_email_addresses": false
          }
        }
        "#);
    }

    #[test]
    fn serialize_command_moderation() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::Moderation(ModerationCommand::Accept(Accept {
                target: ParticipantId::nil(),
            })),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "moderation",
          "payload": {
            "action": "accept",
            "target": "00000000-0000-0000-0000-000000000000"
          }
        }
        "#);
    }

    #[test]
    fn serialize_command_raise_hands() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::RaiseHands(RaiseHandsCommand::RaiseHand),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "raise_hands",
          "payload": {
            "action": "raise_hand"
          }
        }
        "#);
    }
}
