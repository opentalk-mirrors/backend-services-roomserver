// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

opentalk_version::build_info!();

pub(super) fn print_version() {
    println!("{}", build_info::BuildInfo::new())
}
