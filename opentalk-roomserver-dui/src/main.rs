// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use app::RoomServerApp;
use clap::Parser as _;

pub mod app;
mod cli;
pub mod client;
pub mod settings;

fn main() -> eframe::Result {
    env_logger::init();

    let args = cli::Args::parse();
    if args.run_tasks().should_exit() {
        return Ok(());
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_native(
        "OpenTalk RoomServer Developer UI",
        options,
        Box::new(|cc| {
            let app = RoomServerApp::new(cc, args.config.as_deref())?;
            Ok(Box::new(app))
        }),
    )
}
