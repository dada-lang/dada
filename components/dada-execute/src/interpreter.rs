use std::pin::Pin;

use crossbeam::atomic::AtomicCell;
use dada_collections::IndexVec;
use dada_ir::{
    code::{bir, syntax},
    error,
    origin_table::HasOriginIn,
    span::FileSpan,
};
use dada_parse::prelude::*;
use parking_lot::Mutex;
use tokio::io::AsyncWriteExt;

use crate::{error::DiagnosticBuilderExt, moment::Moment};

pub(crate) struct Interpreter<'me> {
    db: &'me dyn crate::Db,

    stdout: tokio::sync::Mutex<Pin<Box<dyn tokio::io::AsyncWrite>>>,

    /// clock tick: increases monotonically
    clock: AtomicCell<u64>,

    /// span of current clock tick
    span: AtomicCell<FileSpan>,

    /// recorded moments in history: occur at significant events
    /// (e.g., when a permission is canceled) so that we can
    /// go back and report errors if needed
    moments: Mutex<IndexVec<Moment, MomentData>>,
}

impl<'me> Interpreter<'me> {
    pub(crate) fn new(
        db: &'me dyn crate::Db,
        stdout: Pin<Box<dyn tokio::io::AsyncWrite>>,
        start_span: FileSpan,
    ) -> Self {
        Self {
            db,
            stdout: tokio::sync::Mutex::new(Box::pin(stdout)),
            clock: Default::default(),
            span: AtomicCell::new(start_span),
            moments: Default::default(),
        }
    }

    pub(crate) fn db(&self) -> &dyn crate::Db {
        self.db
    }

    /// Advance to the next clock tick, potentially altering the current span
    /// in the process.
    pub(crate) fn tick_clock(&self, span: FileSpan) {
        self.clock.fetch_add(1);
        self.span.store(span);
    }

    /// Return the span at the current moment.
    pub(crate) fn span_now(&self) -> FileSpan {
        self.span.load()
    }

    /// Record the current moment for posterity.
    pub(crate) fn moment_now(&self) -> Moment {
        let clock = self.clock.load();
        let mut moments = self.moments.lock();

        if let Some(last_moment) = moments.last() {
            if last_moment.clock == clock {
                return moments.last_key().unwrap();
            }
        }

        let span = self.span.load();
        moments.push(MomentData { clock, span });
        return moments.last_key().unwrap();
    }

    /// Return the span for a recorded moment.
    pub(crate) fn span(&self, moment: Moment) -> FileSpan {
        let moments = self.moments.lock();
        moments[moment].span
    }

    /// Returns the `FileSpan` for a given expression `expr` found in `bir`
    pub(crate) fn span_from_bir(
        &self,
        bir: bir::Bir,
        expr: impl HasOriginIn<bir::Origins, Origin = syntax::Expr>,
    ) -> FileSpan {
        let code = bir.origin(self.db());
        let filename = code.filename(self.db());
        let syntax_expr = bir.origins(self.db())[expr];
        let syntax_tree = code.syntax_tree(self.db());
        syntax_tree.spans(self.db())[syntax_expr].in_file(filename)
    }

    pub(crate) async fn print_bytes(&self, mut text: &[u8]) -> eyre::Result<()> {
        while !text.is_empty() {
            match self.stdout.lock().await.write(text).await {
                Ok(written) => text = &text[written..],
                Err(e) => {
                    let span_now = self.span_now();
                    return Err(error!(span_now, "error printing bytes: {}", e).eyre(self.db()));
                }
            }
        }
        return Ok(());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MomentData {
    clock: u64,
    span: FileSpan,
}
