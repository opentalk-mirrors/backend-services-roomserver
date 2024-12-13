// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use clap::{ArgAction, Parser, Subcommand};

pub(crate) mod openapi;
mod version;

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
    pub(crate) config: String,

    /// Print long version description and exit.
    #[clap(long, action=ArgAction::SetTrue)]
    pub(crate) version: bool,

    #[clap(subcommand)]
    pub(crate) cmd: Option<SubCommand>,
}

impl Args {
    /// Execute potential informational tasks like printing help messages or version info.
    ///
    /// When [`ProgramFlow::Exit`] is returned, the program should exit, otherwise the program should continue.
    pub(crate) fn run_tasks(&self) -> ProgramFlow {
        if self.version {
            version::print_version();
            ProgramFlow::Exit
        } else {
            ProgramFlow::Continue
        }
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
