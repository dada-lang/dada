use crossbeam::atomic::AtomicCell;
use dada_collections::IndexVec;
use dada_ir::span::FileSpan;
use parking_lot::Mutex;

use crate::{kernel::Kernel, moment::Moment};

pub(crate) struct Interpreter<'me> {
    db: &'me dyn crate::Db,

    kernel: &'me dyn Kernel,

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
        kernel: &'me dyn Kernel,
        start_span: FileSpan,
    ) -> Self {
        Self {
            db,
            kernel,
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
        moments.last_key().unwrap()
    }

    /// Return the span for a recorded moment.
    pub(crate) fn span(&self, moment: Moment) -> FileSpan {
        let moments = self.moments.lock();
        moments[moment].span
    }

    pub(crate) fn kernel(&self) -> &dyn Kernel {
        &*self.kernel
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MomentData {
    clock: u64,
    span: FileSpan,
}
