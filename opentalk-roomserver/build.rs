// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Context as _;
use vergen::{BuildBuilder, CargoBuilder, RustcBuilder};
use vergen_gix::{Emitter, GixBuilder};

fn main() -> anyhow::Result<()> {
    let mut emitter = Emitter::default();
    let builder = emitter
        .add_instructions(&CargoBuilder::all_cargo().context("Failed to build cargo variables")?)
        .context("Failed to add cargo instructions")?
        .add_instructions(&BuildBuilder::all_build().context("Failed to build builder variables")?)
        .context("Failed to add builder variables")?
        .add_instructions(&RustcBuilder::all_rustc().context("Failed to build rustc variables")?)
        .context("Failed to add rustc variables")?;

    if is_contained_in_git()? {
        builder
            .add_instructions(&GixBuilder::all_git().context("Failed to build git variables")?)
            .context("Failed to add git variables")?;
    }
    builder.emit().context("Failed to emit")?;

    Ok(())
}

/// Checks whether the current or one of the parent directories contains a `.git` entry.
fn is_contained_in_git() -> anyhow::Result<bool> {
    let current_dir = std::env::current_dir().context("Failed to get current dir")?;
    let mut parents = vec![];
    let mut path = &*current_dir
        .canonicalize()
        .context("Failed to canonicalize path")?;
    parents.push(path);
    while let Some(parent) = path.parent() {
        parents.push(parent);
        path = parent;
    }
    for parent in parents.into_iter().rev() {
        if parent.join(".git").exists() {
            return Ok(true);
        }
    }

    println!("cargo::warning=No .git directory found");
    Ok(false)
}
