// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_common::modules::{ModuleId, module_id};

pub const ECHO_MODULE_ID: ModuleId = module_id!("echo");

pub mod command;
pub mod event;
