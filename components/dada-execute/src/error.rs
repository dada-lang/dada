use dada_ir::diagnostic::{Diagnostic, DiagnosticBuilder};

#[extension_trait::extension_trait]
pub impl DiagnosticBuilderExt for DiagnosticBuilder {
    fn eyre(self) -> eyre::Report {
        eyre::Report::new(DiagnosticError {
            diagnostic: Box::new(self.finish()),
        })
    }
}

#[derive(Debug)]
pub struct DiagnosticError {
    #[allow(dead_code)]
    diagnostic: Box<Diagnostic>,
}

impl std::error::Error for DiagnosticError {}

impl std::fmt::Display for DiagnosticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}") // FIXME
    }
}
