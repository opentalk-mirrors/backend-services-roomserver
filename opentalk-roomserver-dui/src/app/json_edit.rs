// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

pub(crate) fn json_editor(ui: &mut egui::Ui, text: &mut String) {
    let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());

    let mut layouter = |ui: &egui::Ui, buf: &str, wrap_width: f32| {
        let mut layout_job =
            egui_extras::syntax_highlighting::highlight(ui.ctx(), ui.style(), &theme, buf, "json");
        layout_job.wrap.max_width = wrap_width;
        ui.fonts(|f| f.layout_job(layout_job))
    };

    egui::ScrollArea::vertical()
        .max_height(f32::INFINITY)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(text)
                    .font(egui::TextStyle::Monospace)
                    .code_editor()
                    .lock_focus(true)
                    .desired_width(f32::INFINITY)
                    .layouter(&mut layouter),
            );
        });
}
