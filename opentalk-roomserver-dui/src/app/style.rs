// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use egui::{Color32, TextEdit};

pub const SECTION_SPACE_HIGHT: f32 = 20.;

pub trait InvalidInputStyle {
    fn invalid_input_style(self) -> Self;
}

impl InvalidInputStyle for TextEdit<'_> {
    fn invalid_input_style(self) -> Self {
        self.text_color(Color32::BLACK)
            .background_color(Color32::LIGHT_RED)
    }
}
