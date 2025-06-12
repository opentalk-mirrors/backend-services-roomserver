// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_signaling::breakout::BREAKOUT_MODULE_ID;
use opentalk_roomserver_types::breakout::command::BreakoutCommand;
use opentalk_roomserver_types_chat::{CHAT_MODULE_ID, command::ChatCommand};
// reexport commands for easier usage
pub use opentalk_roomserver_types_livekit::{
    LIVEKIT_MODULE_ID, LiveKitCommand, MicrophoneRestrictionState,
};
use opentalk_roomserver_types_ping::{PING_MODULE_ID, command::PingCommand};
use opentalk_types_common::modules::ModuleId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SignalingCommand {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<u64>,
    #[serde(flatten)]
    pub content: SignalingModuleCommand,
}

impl SignalingCommand {
    pub fn namespace(&self) -> ModuleId {
        self.content.namespace()
    }
}

impl From<SignalingModuleCommand> for SignalingCommand {
    fn from(content: SignalingModuleCommand) -> Self {
        Self {
            transaction_id: None,
            content,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "namespace", content = "content", rename_all = "snake_case")]
pub enum SignalingModuleCommand {
    // there are no core commands at the moment
    // Core(),
    Breakout(BreakoutCommand),
    Ping(PingCommand),
    Chat(ChatCommand),

    #[serde(rename = "livekit")]
    LiveKit(LiveKitCommand),
}

impl SignalingModuleCommand {
    pub fn namespace(&self) -> ModuleId {
        match self {
            Self::Breakout(..) => BREAKOUT_MODULE_ID,
            Self::Ping(..) => PING_MODULE_ID,
            Self::Chat(..) => CHAT_MODULE_ID,
            Self::LiveKit(..) => LIVEKIT_MODULE_ID,
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_roomserver_types::breakout::command::BreakoutCommand;
    use opentalk_roomserver_types_chat::command::ChatCommand;
    use opentalk_roomserver_types_livekit::LiveKitCommand;
    use opentalk_roomserver_types_ping::command::PingCommand;
    use opentalk_types_common::modules::ModuleId;
    use serde::Deserialize;

    use super::SignalingModuleCommand;
    use crate::api::command::SignalingCommand;

    #[derive(Debug, Clone, Deserialize)]
    pub struct NamespaceOnly {
        pub namespace: ModuleId,
    }

    #[test]
    fn serialize_command_breakout() {
        let command = SignalingCommand {
            transaction_id: None,
            content: SignalingModuleCommand::Breakout(BreakoutCommand::Stop { delay: None }),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "breakout",
          "content": {
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
            content: SignalingModuleCommand::Chat(ChatCommand::DisableChat),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "chat",
          "content": {
            "action": "disable_chat"
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, command.namespace());
    }

    #[test]
    fn serialize_command_ping() {
        let command = SignalingCommand {
            transaction_id: None,
            content: SignalingModuleCommand::Ping(PingCommand::Ping),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "ping",
          "content": {
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
            content: SignalingModuleCommand::LiveKit(LiveKitCommand::CreateNewAccessToken),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "livekit",
          "content": {
            "action": "create_new_access_token"
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, command.namespace());
    }
}
