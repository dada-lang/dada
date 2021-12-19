use crate::span::FullSpan;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub struct Diagnostic {
    pub severity: Severity,
    pub span: FullSpan,
    pub message: String,
    pub labels: Vec<Label>,
    pub children: Vec<Diagnostic>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Severity {
    Help,
    Note,
    Warning,
    Error,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub struct Label {
    pub span: FullSpan,
    pub message: String,
}

#[salsa::accumulator(in crate::Jar)]
pub struct Diagnostics(Diagnostic);

/// Convenience macro for avoiding `format!`
#[macro_export]
macro_rules! diag {
    ($severity:expr, $span:expr, $($message:tt)*) => {
        $crate::diagnostic::Diagnostic::new($severity, $span, format!($($message)*))
    }
}

/// Convenience macro for avoiding `format!`
#[macro_export]
macro_rules! error {
    ($span:expr, $($message:tt)*) => {
        $crate::diagnostic::Diagnostic::new($crate::diagnostic::Severity::Error, $span, format!($($message)*))
    }
}

/// Convenience macro for avoiding `format!`
#[macro_export]
macro_rules! warning {
    ($span:expr, $($message:tt)*) => {
        $crate::diagnostic::Diagnostic::new($crate::diagnostic::Severity::Warning, $span, format!($($message)*))
    }
}

/// Convenience macro for avoiding `format!`
#[macro_export]
macro_rules! note {
    ($span:expr, $($message:tt)*) => {
        $crate::diagnostic::Diagnostic::new($crate::diagnostic::Severity::Note, $span, format!($($message)*))
    }
}

/// Convenience macro for avoiding `format!`
#[macro_export]
macro_rules! help {
    ($span:expr, $($message:tt)*) => {
        $crate::diagnostic::Diagnostic::new($crate::diagnostic::Severity::Help, $span, format!($($message)*))
    }
}

impl Diagnostic {
    /// Create a new diagnostic with the given "main message" at the
    /// given span.
    pub fn new(severity: Severity, span: FullSpan, message: String) -> DiagnosticBuilder {
        DiagnosticBuilder::new(severity, span, message)
    }

    /// Emit the diagnostic to the [`Diagnostics`] accumulator.
    /// You can fetch the diagnostics produced by a query (and its
    /// dependencies) by invoking `query::accumulated::<Diagnostics>(..)`.
    pub fn emit(self, db: &dyn crate::Db) {
        Diagnostics::push(db, self)
    }
}

impl Label {
    pub fn span(&self) -> FullSpan {
        self.span
    }

    pub fn message(&self) -> &String {
        &self.message
    }
}

pub struct DiagnosticBuilder {
    severity: Severity,
    span: FullSpan,
    message: String,
    labels: Vec<Label>,
    children: Vec<Diagnostic>,
}

impl DiagnosticBuilder {
    fn new(severity: Severity, span: FullSpan, message: String) -> Self {
        Self {
            severity,
            span,
            message,
            labels: vec![],
            children: vec![],
        }
    }

    /// Add a label to this diagnostic
    pub fn label(mut self, span: FullSpan, message: String) -> Self {
        self.labels.push(Label { span, message });
        self
    }

    /// Add a child diagnostic. Our severity is raised to at least
    /// the child's level.
    pub fn child(mut self, diagnostic: Diagnostic) -> Self {
        // Raise our severity to the child's level. Note sure if this
        // is important, it just seems weird to have a "note" with
        // an "error" child.
        self.severity = self.severity.max(diagnostic.severity);

        self.children.push(diagnostic);
        self
    }

    /// Return the completed diagnostic.
    pub fn finish(mut self) -> Diagnostic {
        if self.labels.is_empty() {
            self.labels.push(Label {
                span: self.span,
                message: format!("here"),
            });
        }

        Diagnostic {
            severity: self.severity,
            span: self.span,
            message: self.message,
            labels: self.labels,
            children: self.children,
        }
    }

    /// Finish and emit the diagnostic.
    pub fn emit(self, db: &dyn crate::Db) {
        self.finish().emit(db)
    }
}
