// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::BTreeSet;

use opentalk_roomserver_signaling::signaling_module::InternalCommand;
use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::ParticipantId;

/// Internal LiveKit commands that can be sent by other modules
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiveKitInternal {
    /// Mutes participants
    Mute {
        /// The module that is sending the command
        sending_module: ModuleId,
        /// The participants that should get muted
        participants: BTreeSet<ParticipantId>,
    },
}

impl InternalCommand for LiveKitInternal {}
