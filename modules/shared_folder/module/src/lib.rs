// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! # Shared Folder Module
//!
//! ## Functionality
//!
//! Shares URL and password to access a shared folder. Moderators as provided with a
//! read-write URL while other users receive a read-only URL.
//!
//! This module requires that the [`SharedFolder`] is available in the [`RoomParameters::module_data`].
//! If these are not present, the module won't initialize and stays disabled.
//!
//! [`RoomParameters::module_data`]: opentalk_roomserver_types::room_parameters::RoomParameters::module_data

use opentalk_roomserver_signaling::{
    module_context::ModuleContext,
    signaling_module::{JoinInfo, NoOp, PeerJoinInfoMap, SignalingModule, SignalingModuleInitData},
};
use opentalk_roomserver_types::{
    client_parameters::Role, connection_id::ConnectionId,
    signaling::module_error::SignalingModuleError,
};
use opentalk_roomserver_types_shared_folder::{
    SHARED_FOLDER_MODULE_ID,
    command::SharedFolderCommand,
    event::{SharedFolderError, SharedFolderEvent},
};
use opentalk_types_common::{modules::ModuleId, shared_folders::SharedFolder};
use opentalk_types_signaling::ParticipantId;
pub struct SharedFolderModule {
    state: SharedFolder,
}

impl SignalingModule for SharedFolderModule {
    const NAMESPACE: ModuleId = SHARED_FOLDER_MODULE_ID;

    type Incoming = SharedFolderCommand;

    type Outgoing = SharedFolderEvent;

    type Internal = NoOp;

    type Loopback = ();

    type JoinInfo = SharedFolder;

    type PeerJoinInfo = ();

    type Error = SharedFolderError;

    fn init(init_data: SignalingModuleInitData) -> Option<Self> {
        let shared_folder_state = init_data
            .room_parameters
            .event
            .as_ref()
            .and_then(|e| e.shared_folder.as_ref())
            .cloned();
        match shared_folder_state {
            Some(state) => Some(Self {
                state: state.clone(),
            }),
            None => {
                tracing::debug!("Received no SharedFolder configuration, module disabled");
                None
            }
        }
    }

    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<JoinInfo<Self>, SignalingModuleError<Self::Error>> {
        if ctx.participant_role(participant_id) == Some(Role::Moderator) {
            Ok(JoinInfo {
                join_success: Some(self.state.clone()),
                peer: PeerJoinInfoMap::default(),
                participant_states: PeerJoinInfoMap::default(),
            })
        } else {
            Ok(JoinInfo {
                join_success: Some(self.state.clone().without_write_access()),
                peer: PeerJoinInfoMap::default(),
                participant_states: PeerJoinInfoMap::default(),
            })
        }
    }

    fn on_participant_disconnected(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
        _participant_id: ParticipantId,
        _connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        Ok(())
    }

    fn on_websocket_message(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
        _sender: ParticipantId,
        _connection_id: ConnectionId,
        _content: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        Ok(())
    }
}
