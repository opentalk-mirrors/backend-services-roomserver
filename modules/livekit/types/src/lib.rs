// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling data types for the OpenTalk livekit module.

mod command;
mod credentials;
mod error;
mod event;
mod internal;
mod microphone_restriction_state;
mod moderator_or_module;
mod settings;
mod state;

use opentalk_types_common::modules::{ModuleId, module_id};

pub use crate::{
    command::{LiveKitCommand, UnrestrictedParticipants},
    credentials::Credentials,
    error::LiveKitError,
    event::LiveKitEvent,
    internal::LiveKitInternal,
    microphone_restriction_state::MicrophoneRestrictionState,
    moderator_or_module::ModeratorOrModule,
    settings::LiveKitSettings,
    state::LiveKitState,
};

/// The module id for the signaling module
pub const LIVEKIT_MODULE_ID: ModuleId = module_id!("livekit");
