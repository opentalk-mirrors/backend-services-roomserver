// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use clap::Parser;

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
}
