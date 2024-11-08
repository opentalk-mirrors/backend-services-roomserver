// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

const DEFAULT_LOGGING_DIRECTIVES: &str = "warn,opentalk_roomserver=info";

pub fn init() -> Result<()> {
    // Layer which acts as filter of traces and spans.
    let filter = create_filter();

    // FMT layer prints the trace events into stdout
    let fmt = tracing_subscriber::fmt::Layer::default();

    // Create registry which contains all layers
    let registry = Registry::default().with(filter).with(fmt);

    registry.init();

    Ok(())
}

/// Create the logging filter
///
/// The priority of the different config options is ROOMSERVER_LOG > RUST_LOG > hard-coded defaults.
fn create_filter() -> EnvFilter {
    fn read_env_var(var: &str) -> Option<String> {
        std::env::var(var).ok().filter(|v| !v.is_empty())
    }

    let directives = read_env_var("ROOMSERVER_LOG")
        .or_else(|| read_env_var(EnvFilter::DEFAULT_ENV))
        .unwrap_or(DEFAULT_LOGGING_DIRECTIVES.to_owned());

    EnvFilter::new(directives)
}
