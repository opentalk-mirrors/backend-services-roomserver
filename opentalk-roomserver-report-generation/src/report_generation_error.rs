// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use ecow::EcoVec;
use typst::diag::SourceDiagnostic;

/// An error that can happen during report generation
#[derive(Debug)]
pub enum ReportGenerationError {
    /// Compilation failed
    Compilation {
        /// The warnings that were emitted during compilation
        warnings: EcoVec<SourceDiagnostic>,
    },

    /// Error creating the dump directory
    DumpDirectoryCreation {
        /// The source of the error
        source: std::io::Error,
    },

    /// Error exporting dump file
    DumpFileExport {
        /// The source of the error
        source: std::io::Error,
    },
}

impl From<EcoVec<SourceDiagnostic>> for ReportGenerationError {
    fn from(warnings: EcoVec<SourceDiagnostic>) -> Self {
        Self::Compilation { warnings }
    }
}
