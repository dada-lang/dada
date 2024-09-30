use std::fmt::Display;

use crate::span::{AbsoluteSpan, Span};
use salsa::{Accumulator, Update};

/// Signals that a diagnostic was reported.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct Reported;

/// Signals that this may complete or report a diagnostic.
/// In practice we use this to mean an error.
pub type Errors<T> = Result<T, Reported>;

/// A diagnostic to be reported to the user.
#[salsa::accumulator]
#[derive(PartialEq, Eq, Hash)]
#[must_use]
pub struct Diagnostic {
    /// Level of the message.
    pub level: Level,

    /// Main location of the message.
    pub span: AbsoluteSpan,

    /// Message to be printed.
    pub message: String,

    /// Labels to be included.
    /// Add labels with the `label` helper method.
    pub labels: Vec<DiagnosticLabel>,

    /// Child diagnostics.
    pub children: Vec<Diagnostic>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Level {
    Note,
    Help,
    Info,
    Warning,
    Error,
}

/// A label to be included in the diagnostic.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DiagnosticLabel {
    /// Level of the label.
    pub level: Level,

    /// The span to be labeled.
    /// Must have the same source file as the main diagnostic!
    pub span: AbsoluteSpan,

    /// Message to be printed for the label.
    pub message: String,
}

impl Diagnostic {
    pub fn error<'db>(db: &'db dyn crate::Db, span: Span<'db>, message: impl Display) -> Self {
        Self::new(db, Level::Error, span, message)
    }

    pub fn new<'db>(
        db: &'db dyn crate::Db,
        level: Level,
        span: Span<'db>,
        message: impl Display,
    ) -> Self {
        let message = message.to_string();
        Diagnostic {
            span: span.absolute_span(db),
            level,
            children: vec![],
            message,
            labels: vec![],
        }
    }

    pub fn report(self, db: &dyn crate::Db) -> Reported {
        self.accumulate(db);
        Reported
    }

    pub fn label(
        mut self,
        db: &dyn crate::Db,
        level: Level,
        span: Span,
        message: impl Display,
    ) -> Self {
        let span = span.absolute_span(db);
        assert_eq!(self.span.source_file, span.source_file);
        self.labels.push(DiagnosticLabel {
            level,
            span,
            message: message.to_string(),
        });
        self
    }
}

pub fn report_all(db: &dyn crate::Db, diagnostics: Vec<Diagnostic>) {
    for diagnostic in diagnostics {
        diagnostic.report(db);
    }
}
