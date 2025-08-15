// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_types::{
    breakout::{BREAKOUT_MODULE_ID, event::BreakoutEvent},
    core::CoreEvent,
};
use opentalk_roomserver_types_chat::{CHAT_MODULE_ID, event::ChatEvent};
use opentalk_roomserver_types_meeting_report::{
    MEETING_REPORT_MODULE_ID, event::MeetingReportEvent,
};
use opentalk_roomserver_types_moderation::{MODERATION_MODULE_ID, event::ModerationEvent};
use opentalk_roomserver_types_ping::{PING_MODULE_ID, event::PingEvent};
use opentalk_roomserver_types_polls::{POLLS_MODULE_ID, event::PollsEvent};
use opentalk_roomserver_types_timer::{TIMER_MODULE_ID, TimerEvent};
use opentalk_types_common::modules::{CORE_MODULE_ID, ModuleId};
use serde::{Deserialize, Serialize};
// reexport events for easier usage
pub use {
    opentalk_roomserver_types_e2ee::{E2EE_MODULE_ID, E2eeEvent},
    opentalk_roomserver_types_livekit::{
        Credentials, LIVEKIT_MODULE_ID, LiveKitError, LiveKitEvent, LiveKitState,
    },
    opentalk_roomserver_types_shared_folder::{
        event::{SharedFolder, SharedFolderError},
        {SHARED_FOLDER_MODULE_ID, event::SharedFolderEvent},
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalingEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<u64>,
    #[serde(flatten)]
    pub content: SignalingModuleEvent,
}

impl SignalingEvent {
    pub fn namespace(&self) -> ModuleId {
        self.content.namespace()
    }
}

impl From<SignalingModuleEvent> for SignalingEvent {
    fn from(content: SignalingModuleEvent) -> Self {
        Self {
            transaction_id: None,
            content,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "namespace", content = "content", rename_all = "snake_case")]
pub enum SignalingModuleEvent {
    Core(CoreEvent),
    Breakout(BreakoutEvent),
    Ping(PingEvent),
    Chat(ChatEvent),

    #[serde(rename = "livekit")]
    LiveKit(LiveKitEvent),

    E2ee(E2eeEvent),

    Timer(TimerEvent),
    Polls(PollsEvent),
    SharedFolder(SharedFolderEvent),
    MeetingReport(MeetingReportEvent),
    Moderation(ModerationEvent),
}

impl SignalingModuleEvent {
    pub fn namespace(&self) -> ModuleId {
        match self {
            Self::Core(..) => CORE_MODULE_ID,
            Self::Breakout(..) => BREAKOUT_MODULE_ID,
            Self::Ping(..) => PING_MODULE_ID,
            Self::Chat(..) => CHAT_MODULE_ID,
            Self::LiveKit(..) => LIVEKIT_MODULE_ID,
            Self::E2ee(..) => E2EE_MODULE_ID,
            Self::Timer(..) => TIMER_MODULE_ID,
            Self::Polls(..) => POLLS_MODULE_ID,
            Self::SharedFolder(..) => SHARED_FOLDER_MODULE_ID,
            Self::MeetingReport(..) => MEETING_REPORT_MODULE_ID,
            Self::Moderation(..) => MODERATION_MODULE_ID,
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_roomserver_types::{
        breakout::event::BreakoutEvent, connection_id::ConnectionId, core::CoreEvent,
    };
    use opentalk_roomserver_types_chat::event::{ChatDisabled, ChatEvent};
    use opentalk_roomserver_types_livekit::LiveKitEvent;
    use opentalk_roomserver_types_meeting_report::event::{MeetingReportEvent, PdfAsset};
    use opentalk_roomserver_types_moderation::event::ModerationEvent;
    use opentalk_roomserver_types_ping::event::PingEvent;
    use opentalk_roomserver_types_polls::{
        ChoiceId, PollId,
        command::{Choices, Vote},
        event::PollsEvent,
    };
    use opentalk_roomserver_types_timer::{
        TimerEvent, event::updated_ready_status::UpdatedReadyStatus,
    };
    use opentalk_types_common::{assets::AssetId, modules::ModuleId};
    use opentalk_types_signaling::ParticipantId;
    use serde::Deserialize;

    use super::SignalingModuleEvent;
    use crate::api::event::SignalingEvent;

    #[derive(Debug, Clone, Deserialize)]
    pub struct NamespaceOnly {
        pub namespace: ModuleId,
    }

    #[test]
    fn serialize_event_core() {
        let event = SignalingEvent {
            transaction_id: None,
            content: SignalingModuleEvent::Core(CoreEvent::ParticipantConnected {
                participant_id: ParticipantId::from_u128(0x01),
                connection_id: ConnectionId::from_u128(0x02),
                peer_join_info: Default::default(),
            }),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "core",
          "content": {
            "participant_connected": {
              "participant_id": "00000000-0000-0000-0000-000000000001",
              "connection_id": "00000000-0000-0000-0000-000000000002"
            }
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, event.namespace());
    }

    #[test]
    fn serialize_event_breakout() {
        let event = SignalingEvent {
            transaction_id: None,
            content: SignalingModuleEvent::Breakout(BreakoutEvent::Closed),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "breakout",
          "content": {
            "message": "closed"
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, event.namespace());
    }

    #[test]
    fn serialize_event_chat() {
        let event = SignalingEvent {
            transaction_id: None,
            content: SignalingModuleEvent::Chat(ChatEvent::ChatDisabled(ChatDisabled {
                issued_by: ParticipantId::from_u128(0x01),
            })),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "chat",
          "content": {
            "message": "chat_disabled",
            "issued_by": "00000000-0000-0000-0000-000000000001"
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, event.namespace());
    }

    #[test]
    fn serialize_event_ping() {
        let event = SignalingEvent {
            transaction_id: None,
            content: SignalingModuleEvent::Ping(PingEvent::Pong),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "ping",
          "content": {
            "message": "pong"
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, event.namespace());
    }

    #[test]
    fn serialize_event_livekit() {
        let event = SignalingEvent {
            transaction_id: None,
            content: SignalingModuleEvent::LiveKit(LiveKitEvent::MicrophoneRestrictionsDisabled),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "livekit",
          "content": {
            "message": "microphone_restrictions_disabled"
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, event.namespace());
    }

    #[test]
    fn serialize_event_timer() {
        let event = SignalingEvent {
            transaction_id: None,
            content: SignalingModuleEvent::Timer(TimerEvent::UpdatedReadyStatus(
                UpdatedReadyStatus {
                    participant_id: ParticipantId::nil(),
                    status: true,
                },
            )),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "timer",
          "content": {
            "message": "updated_ready_status",
            "participant_id": "00000000-0000-0000-0000-000000000000",
            "status": true
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, event.namespace());
    }

    #[test]
    fn serialize_event_polls() {
        let event = SignalingEvent {
            transaction_id: None,
            content: SignalingModuleEvent::Polls(PollsEvent::Voted(Vote {
                poll_id: PollId::nil(),
                choices: Choices::Single {
                    choice_id: ChoiceId::from_u32(0),
                },
            })),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "polls",
          "content": {
            "message": "voted",
            "poll_id": "00000000-0000-0000-0000-000000000000",
            "choice_id": 0
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, event.namespace());
    }

    #[test]
    fn serialize_event_meeting_report() {
        let event = SignalingEvent {
            transaction_id: None,
            content: SignalingModuleEvent::MeetingReport(MeetingReportEvent::PdfAsset(PdfAsset {
                filename: "name".into(),
                asset_id: AssetId::nil(),
            })),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "meeting_report",
          "content": {
            "message": "pdf_asset",
            "filename": "name",
            "asset_id": "00000000-0000-0000-0000-000000000000"
          }
        }
        "#);
    }

    #[test]
    fn serialize_event_moderation() {
        let event = SignalingEvent {
            transaction_id: None,
            content: SignalingModuleEvent::Moderation(ModerationEvent::Accepted),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "moderation",
          "content": {
            "message": "accepted"
          }
        }
        "#);
    }
}
