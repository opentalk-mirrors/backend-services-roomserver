// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use ecow::EcoVec;
use thiserror::Error;
use typst::diag::SourceDiagnostic;

/// An error that can happen during report generation
#[derive(Debug, Error)]
pub enum ReportGenerationError {
    /// Compilation failed
    #[error("Compilation failed, the following warnings were emitted: {warnings:#?}")]
    Compilation {
        /// The warnings that were emitted during compilation
        warnings: EcoVec<SourceDiagnostic>,
    },
}

impl From<EcoVec<SourceDiagnostic>> for ReportGenerationError {
    fn from(warnings: EcoVec<SourceDiagnostic>) -> Self {
        Self::Compilation { warnings }
    }
}
