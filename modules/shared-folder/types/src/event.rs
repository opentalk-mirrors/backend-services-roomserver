// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types::signaling::module_error::ModuleError;
pub use opentalk_types_common::shared_folders::SharedFolder;
use opentalk_types_signaling::{ForRole, Role};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SharedFolderError;

impl ModuleError for SharedFolderError {}

/// Events sent out by the `shared_folder` module
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum SharedFolderEvent {
    /// The shared folder data has changed, e.g. by a participant
    /// being promoted to or demoted from moderator role
    Updated(SharedFolder),

    Error(SharedFolderError),
}

impl From<SharedFolderError> for SharedFolderEvent {
    fn from(error: SharedFolderError) -> Self {
        Self::Error(error)
    }
}

impl ForRole for SharedFolderEvent {
    /// Get an equivalent shared folder event, cut down to match the signaling role
    fn for_role(self, role: Role) -> Self {
        match self {
            SharedFolderEvent::Updated(state) => SharedFolderEvent::Updated(state.for_role(role)),
            other => other,
        }
    }
}
