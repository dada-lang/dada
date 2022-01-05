use dada_ir::diagnostic::DiagnosticBuilder;

#[extension_trait::extension_trait]
pub impl DiagnosticBuilderExt for DiagnosticBuilder {
    fn eyre(self, db: &dyn crate::Db) -> eyre::Report {
        let diagnostic = self.finish();
        match dada_error_format::format_diagnostics(db, &[diagnostic]) {
            Ok(string) => eyre::Report::new(DiagnosticError { string }),
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
}

impl std::error::Error for DiagnosticError {}

impl std::fmt::Display for DiagnosticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string)
    }
}
