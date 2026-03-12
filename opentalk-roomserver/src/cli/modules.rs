// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use clap::Subcommand;
use itertools::Itertools;
use opentalk_roomserver_room::signaling::{DescriptionPrinter, ModuleInitializer};
use opentalk_roomserver_signaling::signaling_module::SignalingModuleFeatureDescription;
use opentalk_types_common::modules::ModuleId;

use crate::modules::setup_registry;

#[derive(Subcommand, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum Command {
    /// List available modules and their features
    List,

    /// Print a documentation of the modules (in Markdown)
    PrintDocumentation,
}

pub(crate) fn handle_command(command: Command) {
    let registry = setup_registry();

    match command {
        Command::List => {
            registry.print(&mut ListPrinter);
        }
        Command::PrintDocumentation => {
            registry.print(&mut MarkdownPrinter);
        }
    }
}

struct ListPrinter;

impl DescriptionPrinter for ListPrinter {
    fn print(&mut self, module_initializer: &dyn ModuleInitializer) {
        println!(
            "{}: [{}]",
            module_initializer.module_id(),
            module_initializer
                .feature_descriptions()
                .iter()
                .map(|f| format!("\"{}\"", f.feature_id))
                .join(", ")
        );
    }
}

struct MarkdownPrinter;

impl DescriptionPrinter for MarkdownPrinter {
    fn print(&mut self, module_initializer: &dyn ModuleInitializer) {
        println!(
            "## Module `{}`\n\n{}\n\n### Features\n\n{}",
            module_initializer.module_id(),
            module_initializer.description(),
            generate_features_documentation(
                &module_initializer.module_id(),
                module_initializer.feature_descriptions()
            )
        )
    }
}

fn generate_features_documentation(
    module_id: &ModuleId,
    features: &[SignalingModuleFeatureDescription],
) -> String {
    if features.is_empty() {
        return "This module does not provide any configurable features.\n".to_string();
    }

    format!(
        "The following features can be configured for the module. All features are enabled by default and can be disabled either [by configuration](https://docs.opentalk.eu/admin/controller/advanced/defaults/) or [by tariff](https://docs.opentalk.eu/admin/controller/advanced/tariffs/).\n\n{}\n",
        features
            .iter()
            .map(|feature| generate_feature_documentation(module_id, feature))
            .join("\n\n")
    )
}

fn generate_feature_documentation(
    module_id: &ModuleId,
    feature: &SignalingModuleFeatureDescription,
) -> String {
    format!(
        "#### `{}::{}`\n\n{}",
        module_id, feature.feature_id, feature.description
    )
}
