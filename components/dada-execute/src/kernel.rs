//! The "kernel" is the interface from the interpreter to the outside world.

use std::{cmp::Ordering, sync::Arc};

use dada_ir::{filename::Filename, function::Function, span::FileSpan};
use salsa::DebugWithDb;

use crate::{
    heap_graph::HeapGraph,
    machine::{ProgramCounter, Value},
};

#[async_trait::async_trait]
pub trait Kernel: Send + Sync {
    /// Implementation for the `print` intrinsic, that prints a line of text.
    ///
    /// # Parameters
    ///
    /// * `await_pc` -- the program counter when the thunk was awaited
    /// * `text` -- the string to print
    async fn print(&mut self, await_pc: ProgramCounter, text: &str) -> eyre::Result<()>;

    /// Prints a newline.
    ///
    /// # Parameters
    ///
    /// * `await_pc` -- the program counter when the thunk was awaited
    async fn print_newline(&mut self, await_pc: ProgramCounter) -> eyre::Result<()> {
        self.print(await_pc, "\n").await
    }

    /// Indicates that we have reached the start of a breakpoint expression.
    fn breakpoint_start(
        &mut self,
        db: &dyn crate::Db,
        breakpoint_filename: Filename,
        breakpoint_index: usize,
        generate_heap_graph: &mut dyn FnMut() -> HeapGraph,
    ) -> eyre::Result<()>;

    /// Indicates that we have reached the end of a breakpoint expression.
    fn breakpoint_end(
        &mut self,
        db: &dyn crate::Db,
        breakpoint_filename: Filename,
        breakpoint_index: usize,
        breakpoint_span: FileSpan,
        generate_heap_graph: &mut dyn FnMut() -> HeapGraph,
    ) -> eyre::Result<()>;
}

#[derive(Default)]
pub struct BufferKernel {
    stop_at_breakpoint: bool,
    breakpoint_callback: Option<BreakpointCallback>,
    track_output_ranges: bool,

    /// Collects the output of the program.
    buffer: String,

    /// Tracks which program counter is responsible for which output.
    buffer_pcs: Vec<OutputRange>,

    /// When we start a breakpoint, we push an entry here.
    started_breakpoints: Vec<(Filename, usize, HeapGraph)>,

    /// When we end a breakpoint, we construct a `BreakpointHeapGraph` and
    /// either invoke `breakpoint_callback` or else buffer it here.
    heap_graphs: Vec<BreakpointRecord>,
}

#[derive(Copy, Clone, Debug)]
pub struct OutputRange {
    pub start: usize,
    pub end: usize,
    pub await_pc: ProgramCounter,
}

impl OutputRange {
    pub fn range(&self) -> std::ops::Range<usize> {
        self.start..self.end
    }
}

pub struct BreakpointRecord {
    pub breakpoint_filename: Filename,
    pub breakpoint_index: usize,
    pub breakpoint_span: FileSpan,
    pub heap_at_start: HeapGraph,
    pub heap_at_end: HeapGraph,
}

impl BreakpointRecord {
    pub fn to_graphviz(&self, db: &dyn crate::Db) -> String {
        self.heap_at_start
            .graphviz_paired(db, false, &self.heap_at_end)
    }
}

type BreakpointCallback =
    Arc<dyn Fn(&dyn crate::Db, &mut BufferKernel, BreakpointRecord) + Send + Sync>;

impl BufferKernel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder method: if `stop_at_breakpoint` is true, then when a breakpoint
    /// is encountered we will stop with the error [`BreakpointExpressionEncountered`].
    /// If false, execution will continue (and the breakpoint may be hit more than
    /// once).
    pub fn stop_at_breakpoint(self, stop_at_breakpoint: bool) -> Self {
        Self {
            stop_at_breakpoint,
            ..self
        }
    }

    /// Builder method: if `track_output_ranges` is true, then track the program
    /// counter from each piece of text that is emitted. These can be inspected
    /// later to determine the bytes at a given output.
    pub fn track_output_ranges(self, track_output_ranges: bool) -> Self {
        Self {
            track_output_ranges,
            ..self
        }
    }

    /// Builder method: invoke the given callback instead of accumulating the
    /// heap graph.
    pub fn breakpoint_callback(
        self,
        callback: impl Fn(&dyn crate::Db, &mut Self, BreakpointRecord) + Send + Sync + 'static,
    ) -> Self {
        Self {
            breakpoint_callback: Some(Arc::new(callback)),
            ..self
        }
    }

    pub async fn interpret(
        &mut self,
        db: &dyn crate::Db,
        function: Function,
        arguments: Vec<Value>,
    ) -> eyre::Result<()> {
        crate::run::interpret(function, db, self, arguments).await
    }

    pub async fn interpret_and_buffer(
        &mut self,
        db: &dyn crate::Db,
        function: Function,
        arguments: Vec<Value>,
    ) {
        match crate::run::interpret(function, db, self, arguments).await {
            Ok(()) => {}
            Err(e) => {
                self.append(&e.to_string());
            }
        }
    }

    /// Take the recorded data from breakpoints that triggered.
    /// This vec will be empty if there is a breakpoint callback set.
    pub fn take_recorded_breakpoints(&mut self) -> Vec<BreakpointRecord> {
        std::mem::take(&mut self.heap_graphs)
    }

    /// Borrow the buffered output.
    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    /// Convert the buffer into the output
    pub fn take_buffer(&mut self) -> String {
        self.buffer_pcs.clear();
        std::mem::take(&mut self.buffer)
    }

    /// Append text into the output buffer
    pub fn append(&mut self, s: &str) {
        self.buffer.push_str(s);
    }

    /// Append text into the output buffer
    fn append_range(&mut self, range: OutputRange) {
        if !self.track_output_ranges {
            return;
        }

        assert!(range.start <= range.end);
        assert!(range.end <= self.buffer.len());

        // No text.
        if range.start == range.end {
            return;
        }

        // If this is appending more text from the same position, then just grow the
        // last range we pushed.
        if let Some(last_range) = self.buffer_pcs.last_mut() {
            assert!(range.start >= last_range.end);

            if last_range.await_pc == range.await_pc
                && last_range.end == range.start
                && last_range.end < range.end
            {
                last_range.end = range.end;
                return;
            }
        }

        // Else push a new range.
        self.buffer_pcs.push(range);
    }

    /// Returns the `OutputRange` tracking who generated the output at the given
    /// offset (along with the full range of text that was generated).
    ///
    /// Returns `None` if we are not tracking that information
    /// or if the text was generated via a call to `append` or other means,
    /// and not from the program.
    pub fn pc_at_offset(&self, offset: usize) -> Option<OutputRange> {
        match self.buffer_pcs.binary_search_by(|range| {
            if (range.start..range.end).contains(&offset) {
                Ordering::Equal
            } else {
                offset.cmp(&range.start)
            }
        }) {
            Ok(index) => Some(self.buffer_pcs[index]),
            Err(_index) => None,
        }
    }

    /// Returns an iterator over the text emitted at a given point along with
    /// the program counter responsible (if available).
    pub fn buffer_with_pcs(&self) -> impl Iterator<Item = (&str, Option<ProgramCounter>)> {
        let mut offset = 0;
        let mut buffer_pcs = self.buffer_pcs.iter().peekable();
        std::iter::from_fn(move || match buffer_pcs.peek() {
            Some(range) if offset < range.start => {
                let text = &self.buffer[offset..range.start];
                offset = range.start;
                Some((text, None))
            }

            Some(range) => {
                assert!(offset == range.start);
                let text = &self.buffer[range.start..range.end];
                let await_pc = range.await_pc;
                offset = range.end;
                buffer_pcs.next();
                Some((text, Some(await_pc)))
            }

            None if offset < self.buffer.len() => {
                let text = &self.buffer[offset..];
                offset = self.buffer.len();
                Some((text, None))
            }

            None => None,
        })
    }
}

#[derive(thiserror::Error, Debug)]
#[error("breakpoint expression encountered")]
pub struct BreakpointExpressionEncountered;

#[async_trait::async_trait]
impl Kernel for BufferKernel {
    async fn print(&mut self, await_pc: ProgramCounter, message: &str) -> eyre::Result<()> {
        let start = self.buffer.len();
        self.append(message);
        let end = self.buffer.len();

        self.append_range(OutputRange {
            start,
            end,
            await_pc,
        });

        Ok(())
    }

    fn breakpoint_start(
        &mut self,
        db: &dyn crate::Db,
        filename: Filename,
        index: usize,
        generate_heap_graph: &mut dyn FnMut() -> HeapGraph,
    ) -> eyre::Result<()> {
        tracing::debug!(
            "breakpoint_start(filename={:?}, index={:?})",
            filename.debug(db),
            index
        );
        let tuple = (filename, index, generate_heap_graph());
        self.started_breakpoints.push(tuple);
        Ok(())
    }

    fn breakpoint_end(
        &mut self,
        db: &dyn crate::Db,
        filename: Filename,
        index: usize,
        span: FileSpan,
        generate_heap_graph: &mut dyn FnMut() -> HeapGraph,
    ) -> eyre::Result<()> {
        tracing::debug!(
            "breakpoint_end(filename={:?}, index={:?})",
            filename.debug(db),
            index
        );

        let (breakpoint_filename, breakpoint_index, heap_at_start) =
            self.started_breakpoints.pop().unwrap();
        assert_eq!(filename, breakpoint_filename);
        assert_eq!(index, breakpoint_index);
        let breakpoint_record = BreakpointRecord {
            breakpoint_filename,
            breakpoint_index,
            breakpoint_span: span,
            heap_at_start,
            heap_at_end: generate_heap_graph(),
        };

        if let Some(cb) = &self.breakpoint_callback {
            let cb = cb.clone();
            cb(db, self, breakpoint_record);
        } else {
            self.heap_graphs.push(breakpoint_record);
        }

        if self.stop_at_breakpoint {
            return Err(BreakpointExpressionEncountered.into());
        }

        Ok(())
    }
}
