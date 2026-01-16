// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::SignalingModuleFrontendData;
use serde::{Deserialize, Serialize};

use crate::{
    MODERATION_MODULE_ID,
    state::{ChangeDisplayNameRestrictionState, ModeratorJoinInfo},
};

/// The state of the `moderation` module.
///
/// This struct is sent to the participant in the `join_success` message
/// when they join successfully to the meeting.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModerationState {
    /// Moderation module data that is only available for moderators
    #[serde(flatten)]
    pub moderator_data: Option<ModeratorJoinInfo>,
    pub display_name_change_restrictions: ChangeDisplayNameRestrictionState,
}

impl SignalingModuleFrontendData for ModerationState {
    const NAMESPACE: Option<ModuleId> = Some(MODERATION_MODULE_ID);
}
