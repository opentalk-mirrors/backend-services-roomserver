// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use egui::{Color32, RichText, TextEdit};

pub const SECTION_SPACE_HIGHT: f32 = 20.;

const INVALID_TEXT_COLOR: Color32 = Color32::BLACK;
const INVALID_BACKGROUND_COLOR: Color32 = Color32::LIGHT_RED;

pub trait InvalidInputStyle {
    fn invalid_input_style(self) -> Self;
}

impl InvalidInputStyle for TextEdit<'_> {
    fn invalid_input_style(self) -> Self {
        self.text_color(INVALID_TEXT_COLOR)
            .background_color(INVALID_BACKGROUND_COLOR)
    }
}

impl InvalidInputStyle for RichText {
    fn invalid_input_style(self) -> Self {
        self.color(INVALID_TEXT_COLOR)
            .background_color(INVALID_BACKGROUND_COLOR)
    }
}
