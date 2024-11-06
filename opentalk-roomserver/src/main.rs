// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>

//! This crate builds an executable that runs the RoomServer. It implements the [_OpenTalk RoomServer Web API_][opentalk_roomserver_web_api].

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Args, SubCommand};
use settings::Settings;
use std::sync::Arc;

mod api;
mod cli;
mod room;
pub(crate) mod settings;
mod trace;

async fn run_web_server(config_file_name: &str) -> Result<()> {
    let settings = Arc::new(Settings::load(config_file_name)?);

    trace::init().context("Failed to initialize tracing")?;

    api::run_web_server(settings).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.cmd {
        Some(SubCommand::Openapi(command)) => {
            cli::openapi::handle_command(command).await?;
        }
        None => run_web_server(&args.config).await?,
    }

    Ok(())
}
