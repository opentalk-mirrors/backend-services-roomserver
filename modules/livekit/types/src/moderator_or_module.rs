// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

/// A moderator or a module
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModeratorOrModule {
    /// A moderator
    Moderator {
        /// The participant id of the moderator
        moderator: ParticipantId,
    },
    /// A module
    Module {
        /// The namespace of the module
        module: ModuleId,
    },
}

impl From<ParticipantId> for ModeratorOrModule {
    fn from(moderator: ParticipantId) -> Self {
        ModeratorOrModule::Moderator { moderator }
    }
}

impl From<ModuleId> for ModeratorOrModule {
    fn from(module: ModuleId) -> Self {
        ModeratorOrModule::Module { module }
    }
}
