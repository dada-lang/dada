use dada_ir::diagnostic::{Diagnostic, DiagnosticBuilder};

#[extension_trait::extension_trait]
pub impl DiagnosticBuilderExt for DiagnosticBuilder {
    fn eyre(self, db: &dyn crate::Db) -> eyre::Report {
        let diagnostic = self.finish();
        match dada_error_format::format_diagnostics(db, &[diagnostic.clone()]) {
            Ok(string) => eyre::Report::new(DiagnosticError { string, diagnostic }),
            Err(report) => {
                // FIXME: should give causal information
                report
            }
        }
    }
}

#[derive(Debug)]
pub struct DiagnosticError {
    string: String,
    diagnostic: Diagnostic,
}

impl DiagnosticError {
    pub fn diagnostic(&self) -> &Diagnostic {
        &self.diagnostic
    }
}

impl std::error::Error for DiagnosticError {}

impl std::fmt::Display for DiagnosticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string)
    }
}
