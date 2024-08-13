// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use clap::{Parser, Subcommand};

pub(crate) mod openapi;

#[derive(Parser, Debug, Clone)]
#[clap(name = "opentalk-roomserver")]
#[command(version, about)]
pub(crate) struct Args {
    #[clap(
        short,
        long,
        default_value = "config.toml",
        help = "Specify path to configuration file"
    )]
    pub(crate) config: String,

    #[clap(subcommand)]
    pub(crate) cmd: Option<SubCommand>,
}

#[derive(Subcommand, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
#[allow(clippy::large_enum_variant)]
pub(crate) enum SubCommand {
    /// OpenAPI related commands
    #[clap(subcommand)]
    Openapi(openapi::Command),
}
