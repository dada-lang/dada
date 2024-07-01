use std::fmt::Display;

use crate::span::{AbsoluteSpan, Span};

#[salsa::accumulator]
pub struct Diagnostics(Diagnostic);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[must_use]
pub struct Diagnostic {
    level: Level,
    span: AbsoluteSpan,
    message: String,
    labels: Vec<DiagnosticLabel>,
    children: Vec<Diagnostic>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Level {
    Note,
    Warning,
    Info,
    Help,
    Error,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct DiagnosticLabel {
    span: Span<'static>,
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

    pub fn report(self, db: &dyn crate::Db) {
        Diagnostics::push(db, self)
    }
}

pub fn report_all(db: &dyn crate::Db, diagnostics: Vec<Diagnostic>) {
    for diagnostic in diagnostics {
        diagnostic.report(db);
    }
}
