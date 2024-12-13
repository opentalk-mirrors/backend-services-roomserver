// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::fmt::Display;

fn profile_human() -> &'static str {
    let profile = env!("VERGEN_CARGO_OPT_LEVEL");

    // Copied from https://doc.rust-lang.org/cargo/reference/profiles.html#opt-level
    match profile {
        "0" => "0, no optimizations",
        "1" => "1, basic optimizations",
        "2" => "2, some optimizations",
        "3" => "3, all optimizations",
        "s" => "'s', optimize for binary size",
        "z" => "'z', optimize for binary size, but also turn off loop vectorization.",
        profile => profile,
    }
}

struct BuildInfo {
    build_timestamp: &'static str,
    build_version: &'static str,
    commit_sha: Option<&'static str>,
    commit_dirty: Option<&'static str>,
    commit_date: Option<&'static str>,
    commit_branch: Option<&'static str>,
    rustc_version: &'static str,
    rustc_channel: &'static str,
    rustc_host_triple: &'static str,
    cargo_target_triple: &'static str,
    cargo_profile: &'static str,
}

impl BuildInfo {
    fn new() -> Self {
        Self {
            build_timestamp: env!("VERGEN_BUILD_TIMESTAMP"),
            build_version: env!("CARGO_PKG_VERSION"),
            commit_sha: option_env!("VERGEN_GIT_SHA"),
            commit_dirty: option_env!("VERGEN_GIT_DIRTY"),
            commit_date: option_env!("VERGEN_GIT_COMMIT_TIMESTAMP"),
            commit_branch: option_env!("VERGEN_GIT_BRANCH"),
            rustc_version: env!("VERGEN_RUSTC_SEMVER"),
            rustc_channel: env!("VERGEN_RUSTC_CHANNEL"),
            rustc_host_triple: env!("VERGEN_RUSTC_HOST_TRIPLE"),
            cargo_target_triple: env!("VERGEN_CARGO_TARGET_TRIPLE"),
            cargo_profile: profile_human(),
        }
    }
}

impl Display for BuildInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Build Timestamp: {}", self.build_timestamp)?;
        writeln!(f, "Build Version: {}", self.build_version)?;
        if let Some(sha) = self.commit_sha {
            write!(f, "Commit SHA: {}", sha)?;
            match self.commit_dirty {
                Some("true") => writeln!(f, "-dirty")?,
                _ => writeln!(f)?,
            }
        }
        if let Some(commit_date) = self.commit_date {
            writeln!(f, "Commit Date: {}", commit_date)?;
        }

        if let Some(commit_branch) = self.commit_branch {
            writeln!(f, "Commit Branch: {}", commit_branch)?;
        }
        writeln!(f, "Rustc Version: {}", self.rustc_version)?;
        writeln!(f, "Rustc Channel: {}", self.rustc_channel)?;
        writeln!(f, "Rustc Host Triple: {}", self.rustc_host_triple)?;
        writeln!(f, "Cargo Target Triple: {}", self.cargo_target_triple)?;
        write!(f, "Cargo Profile: {}", self.cargo_profile)
    }
}

pub(super) fn print_version() {
    println!("{}", BuildInfo::new())
}
