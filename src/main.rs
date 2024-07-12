// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>

use anyhow::{Context, Result};
use clap::Parser;
use cli::Args;
use settings::Settings;
use std::sync::Arc;

mod api;
mod cli;
mod room;
pub(crate) mod settings;
mod trace;
mod types;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let settings = Arc::new(Settings::load(&args.config)?);

    trace::init().context("Failed to initialize tracing")?;

    api::run_web_server(settings).await?;

    Ok(())
}
