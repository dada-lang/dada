use crate::span::{FileSpan, Span};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub struct Diagnostic {
    pub severity: Severity,
    pub span: FileSpan,
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
    pub span: FileSpan,
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
    pub fn new(severity: Severity, span: FileSpan, message: String) -> DiagnosticBuilder {
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
    pub fn span(&self) -> FileSpan {
        self.span
    }

    pub fn message(&self) -> &String {
        &self.message
    }
}

#[must_use]
pub struct DiagnosticBuilder {
    severity: Severity,
    span: FileSpan,
    message: String,
    labels: Vec<Label>,
    children: Vec<Diagnostic>,
}

impl DiagnosticBuilder {
    fn new(severity: Severity, span: FileSpan, message: impl ToString) -> Self {
        Self {
            severity,
            span,
            message: message.to_string(),
            labels: vec![],
            children: vec![],
        }
    }

    /// Add a label to this diagnostic; the label is assumed to
    /// be in the same file as the "main" error.
    pub fn label(self, span: Span, message: impl ToString) -> Self {
        let file_span = span.in_file(self.span.filename);
        self.label_in_any_file(file_span, message)
    }

    /// Add a label to this diagnostic; the label may be in any file.
    pub fn label_in_any_file(mut self, span: FileSpan, message: impl ToString) -> Self {
        self.labels.push(Label {
            span,
            message: message.to_string(),
        });
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
            let span = self.span;
            self = self.label_in_any_file(span, "here");
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
