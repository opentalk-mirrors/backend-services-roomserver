// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_common::modules::{ModuleId, module_id};

/// The module id for the signaling module
pub const MODULE_ID: ModuleId = module_id!("chat");

pub mod command;
pub mod event;
