// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::path::PathBuf;

use build_info::BuildInfo;
use clap::{Parser, Subcommand};
use opentalk_version::InfoArgs;

mod license;
pub(crate) mod openapi;

opentalk_version::build_info!();

/// Whether the program should start or exit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProgramFlow {
    /// Exit the program.
    Exit,

    /// The program should continue execution.
    Continue,
}

impl ProgramFlow {
    /// Returns `true` if the program flow is [`Exit`].
    ///
    /// [`Exit`]: ProgramFlow::Exit
    #[must_use]
    pub(crate) fn should_exit(&self) -> bool {
        matches!(self, Self::Exit)
    }
}

#[derive(Parser, Debug, Clone)]
#[clap(name = "opentalk-roomserver")]
#[command(about)]
pub(crate) struct Args {
    #[clap(
        short,
        long,
        default_value = "config.toml",
        help = "Specify path to configuration file"
    )]
    pub(crate) config: PathBuf,

    #[command(flatten)]
    pub(crate) info: InfoArgs,

    #[clap(subcommand)]
    pub(crate) cmd: Option<SubCommand>,
}

impl Args {
    /// Execute potential informational tasks like printing help messages or version info.
    ///
    /// When [`ProgramFlow::Exit`] is returned, the program should exit, otherwise the program should continue.
    pub(crate) fn run_tasks(&self) -> ProgramFlow {
        if !self.info.should_print() {
            return ProgramFlow::Continue;
        }
        let build_info = BuildInfo::with_license(license::LICENSE.to_owned());
        if let Some(text) = build_info.format(&self.info) {
            println!("{text}");
        }
        ProgramFlow::Exit
    }
}

#[derive(Subcommand, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
#[allow(clippy::large_enum_variant)]
pub(crate) enum SubCommand {
    /// OpenAPI related commands
    #[clap(subcommand)]
    Openapi(openapi::Command),
}
