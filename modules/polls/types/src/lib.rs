// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling data types for the OpenTalk polls module.

#![deny(
    bad_style,
    missing_debug_implementations,
    missing_docs,
    overflowing_literals,
    patterns_in_fns_without_body,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]

pub mod command;
pub mod event;
pub mod state;

mod choice;
mod choice_id;
mod item;
mod poll_id;
mod results;

pub use choice::Choice;
pub use choice_id::ChoiceId;
pub use item::Item;
use opentalk_types_common::modules::{ModuleId, module_id};
pub use poll_id::PollId;
pub use results::Results;

/// The module id for the signaling module
pub const POLLS_MODULE_ID: ModuleId = module_id!("polls");
