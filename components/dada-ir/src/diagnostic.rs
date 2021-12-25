use crate::{
    filename::Filename,
    span::{FileSpan, Span},
};

/// Used as the "error" value for a `Result` to indicate that an error was detected
/// and reported to the user (i.e., pushed onto the [`Diagnostics`] accumulator).
pub struct ErrorReported;

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
    pub fn emit(self, db: &dyn crate::Db) -> ErrorReported {
        Diagnostics::push(db, self);
        ErrorReported
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
    pub fn label(mut self, span: impl IntoFileSpan, message: impl ToString) -> Self {
        let span = span.maybe_in_file(self.span.filename);
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
            self = self.label(span, "here");
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
    pub fn emit(self, db: &dyn crate::Db) -> ErrorReported {
        self.finish().emit(db)
    }
}

pub trait IntoFileSpan {
    fn maybe_in_file(self, default_file: Filename) -> FileSpan;
}

impl IntoFileSpan for FileSpan {
    fn maybe_in_file(self, _default_file: Filename) -> FileSpan {
        self
    }
}

impl IntoFileSpan for Span {
    fn maybe_in_file(self, default_file: Filename) -> FileSpan {
        self.in_file(default_file)
    }
}
