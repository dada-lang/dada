use crate::span::FullSpan;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Diagnostic {
    span: FullSpan,
    message: String,
}

#[salsa::accumulator(in crate::Jar)]
pub struct Diagnostics(Diagnostic);

/// Convenience macro for avoiding `format!`
#[macro_export]
macro_rules! diag {
    ($span:expr, $($message:tt)*) => {
        $crate::diagnostic::Diagnostic::new($span, format!($($message)*))
    }
}

impl Diagnostic {
    /// Create a new diagnostic with the given "main message" at the
    /// given span.
    pub fn new(span: FullSpan, message: String) -> DiagnosticBuilder {
        DiagnosticBuilder::new(span, message)
    }

    /// Return the "main span" for this message
    pub fn span(&self) -> FullSpan {
        self.span
    }

    /// Return the "main message" for this message
    pub fn message(&self) -> &String {
        &self.message
    }

    /// Emit the diagnostic to the [`Diagnostics`] accumulator.
    /// You can fetch the diagnostics produced by a query (and its
    /// dependencies) by invoking `query::accumulated::<Diagnostics>(..)`.
    pub fn emit(self, db: &dyn crate::Db) {
        Diagnostics::push(db, self)
    }
}

pub struct DiagnosticBuilder {
    span: FullSpan,
    message: String,
}

impl DiagnosticBuilder {
    fn new(span: FullSpan, message: String) -> Self {
        Self { span, message }
    }

    /// Return the completed diagnostic.
    pub fn finish(self) -> Diagnostic {
        Diagnostic {
            span: self.span,
            message: self.message,
        }
    }

    /// Finish and emit the diagnostic.
    pub fn emit(self, db: &dyn crate::Db) {
        self.finish().emit(db)
    }
}
