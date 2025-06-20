// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_common::modules::{ModuleId, module_id};

pub const PING_MODULE_ID: ModuleId = module_id!("ping");

pub mod command;
pub mod error;
pub mod event;
