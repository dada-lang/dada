use crossbeam::atomic::AtomicCell;
use dada_collections::IndexVec;
use dada_ir::span::FileSpan;
use parking_lot::Mutex;

use crate::moment::Moment;

pub struct Interpreter<'me> {
    db: &'me dyn crate::Db,
    clock: AtomicCell<u64>,
    moments: Mutex<IndexVec<Moment, MomentData>>,
}

pub struct StackFrameClock(u64);

impl Interpreter<'_> {
    pub fn db(&self) -> &dyn crate::Db {
        self.db
    }

    pub fn tick_clock(&self) {
        self.clock.fetch_add(1);
    }

    pub fn record_moment(&self, span: FileSpan) -> Moment {
        let clock = self.clock.load();
        let moments = self.moments.lock();

        let data = MomentData { clock: clock, span };
        if let Some(last_moment) = moments.last() {
            if *last_moment == data {
                return moments.last_key().unwrap();
            }
        }

        moments.push(data);
        return moments.last_key().unwrap();
    }

    pub fn span(&self, moment: Moment) -> FileSpan {
        let moments = self.moments.lock();
        moments[moment].span
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MomentData {
    clock: u64,
    span: FileSpan,
}
