use std::fmt::Display;

use crate::{span::{AbsoluteSpan, Span}, DebugEvent, DebugEventPayload};
use dada_util::debug;
use salsa::{Accumulator, Update};

mod render;

/// Signals that a diagnostic was reported at the given span.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct Reported(pub AbsoluteSpan);

impl Reported {
    pub fn span<'db>(self, db: &'db dyn crate::Db) -> Span<'db> {
        self.0.into_span(db)
    }
}

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

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, Hash)]
pub struct RenderOptions {
    pub no_color: bool,
}

impl Diagnostic {
    pub fn error<'db>(db: &'db dyn crate::Db, span: Span<'db>, message: impl Display) -> Self {
        Self::new(db, Level::Error, span, message)
    }

    pub fn help<'db>(db: &'db dyn crate::Db, span: Span<'db>, message: impl Display) -> Self {
        Self::new(db, Level::Help, span, message)
    }

    pub fn info<'db>(db: &'db dyn crate::Db, span: Span<'db>, message: impl Display) -> Self {
        Self::new(db, Level::Info, span, message)
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
        debug!("reporting diagnostic", self);
        let span = self.span;

        if let Some(debug_tx) = db.debug_tx() {
            debug_tx.send(DebugEvent {
                url: span.source_file.url(db).clone(),
                start: span.start,
                end: span.end,
                payload: DebugEventPayload::Diagnostic(self.clone()),
            }).unwrap();
        }

        self.accumulate(db);

        Reported(span)
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

    pub fn child(mut self, child: Diagnostic) -> Self {
        self.children.push(child);
        self
    }

    pub fn render(&self, db: &dyn crate::Db, opts: &RenderOptions) -> String {
        render::render(db, opts, self)
    }
}

pub fn report_all(db: &dyn crate::Db, diagnostics: Vec<Diagnostic>) {
    for diagnostic in diagnostics {
        diagnostic.report(db);
    }
}

pub fn ordinal(n: usize) -> impl std::fmt::Display {
    match n % 10 {
        1 => format!("{}st", n),
        2 => format!("{}nd", n),
        3 => format!("{}rd", n),
        _ => format!("{}th", n),
    }
}

/// Many of our types have some value that represents an error in the input.
pub trait Err<'db> {
    fn err(db: &'db dyn crate::Db, reported: Reported) -> Self;
}

impl<'db, T> Err<'db> for Errors<T> {
    fn err(_db: &'db dyn crate::Db, reported: Reported) -> Self {
        Err(reported)
    }
}

impl<'db> Err<'db> for Reported {
    fn err(_db: &'db dyn crate::Db, reported: Reported) -> Self {
        reported
    }
}
