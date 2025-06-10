// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_signaling::signaling_module::{
    CreateReplica, JoinInfo, SignalingModule, SignalingModuleInitData,
};
use opentalk_roomserver_types::signaling::module_error::{ModuleError, SignalingModuleError};
use opentalk_types_common::modules::{ModuleId, module_id};
use serde::{Deserialize, Serialize};

pub struct MockModule {}

#[derive(Serialize, Deserialize)]
pub enum Command {
    Valid,
    Invalid,
}
impl CreateReplica<Event> for Command {
    fn replicate(&self) -> Option<Event> {
        None
    }
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Event {
    Success,
    Error,
}

impl From<Error> for Event {
    fn from(_: Error) -> Self {
        Event::Error
    }
}

#[derive(Debug)]
pub struct Error;

impl ModuleError for Error {}

impl SignalingModule for MockModule {
    const NAMESPACE: ModuleId = module_id!("mock");

    type Incoming = Command;

    type Outgoing = Event;

    type Loopback = ();

    type JoinInfo = ();

    type PeerJoinInfo = ();

    type Error = Error;

    fn init(_init_data: SignalingModuleInitData) -> Option<Self> {
        Some(Self {})
    }

    #[allow(unused_variables)]
    fn on_participant_joined(
        &mut self,
        ctx: &mut opentalk_roomserver_signaling::module_context::ModuleContext<'_, Self>,
        participant_id: opentalk_types_signaling::ParticipantId,
        connection_id: opentalk_roomserver_types::connection_id::ConnectionId,
        is_first_connection: bool,
    ) -> Result<JoinInfo<Self>, SignalingModuleError<Self::Error>> {
        Ok(JoinInfo::default())
    }

    #[allow(unused_variables)]
    fn on_participant_disconnected(
        &mut self,
        ctx: &mut opentalk_roomserver_signaling::module_context::ModuleContext<'_, Self>,
        participant_id: opentalk_types_signaling::ParticipantId,
        connection_id: opentalk_roomserver_types::connection_id::ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn on_websocket_message(
        &mut self,
        ctx: &mut opentalk_roomserver_signaling::module_context::ModuleContext<'_, Self>,
        sender: opentalk_types_signaling::ParticipantId,
        connection_id: opentalk_roomserver_types::connection_id::ConnectionId,
        content: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match content {
            Command::Valid => {
                ctx.send_ws_message([sender], Event::Success).unwrap();
                Ok(())
            }
            Command::Invalid => Err(Error.into()),
        }
    }

    #[allow(unused_variables)]
    fn on_loopback_event(
        &mut self,
        ctx: &mut opentalk_roomserver_signaling::module_context::ModuleContext<'_, Self>,
        event: Self::Loopback,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        Ok(())
    }
}
