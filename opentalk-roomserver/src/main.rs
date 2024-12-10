// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>

//! This crate builds an executable that runs the RoomServer. It implements the [_OpenTalk RoomServer Web API_][opentalk_roomserver_web_api].

use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Args, SubCommand};
use service_probe::start_probe;
use settings::Settings;

mod api;
mod cli;
#[cfg(test)]
mod mocking;
mod room;
pub(crate) mod settings;
mod trace;

async fn run_web_server(config_file_name: &str) -> Result<()> {
    let settings = Arc::new(Settings::load(config_file_name)?);

    trace::init().context("Failed to initialize tracing")?;
    if let Some(monitoring) = &settings.monitoring {
        start_probe(monitoring.addr, monitoring.port)
            .await
            .context("Failed to start monitoring endpoint")?;
    }
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
