// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use egui::{Key, KeyboardShortcut, Modifiers};

pub const EXIT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Q);
pub const ERROR_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::E);
pub const SETTINGS_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Comma);
pub const DELETE_MODE_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::R);

pub const BACK_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::L);

pub const SEND_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Enter);
pub const SUBMIT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Enter);
pub const DISCONNECT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::D);
pub const CLEAR_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::L);

pub const PREVIOUS_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL, Key::ArrowUp);
pub const SUCCESSOR_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL, Key::ArrowDown);
pub const TOGGLE_HISTORY_PANEL_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL, Key::H);
