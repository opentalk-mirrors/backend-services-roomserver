// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling data types for the OpenTalk livekit module.

#![deny(
    bad_style,
    missing_debug_implementations,
    missing_docs,
    overflowing_literals,
    patterns_in_fns_without_body,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code
)]

mod command;
mod credentials;
mod error;
mod event;
mod microphone_restriction_state;
mod state;

use opentalk_types_common::modules::{ModuleId, module_id};

pub use crate::{
    command::{LiveKitCommand, UnrestrictedParticipants},
    credentials::Credentials,
    error::LiveKitError,
    event::LiveKitEvent,
    microphone_restriction_state::MicrophoneRestrictionState,
    state::LiveKitState,
};

/// The module id for the signaling module
pub const LIVEKIT_MODULE_ID: ModuleId = module_id!("livekit");
