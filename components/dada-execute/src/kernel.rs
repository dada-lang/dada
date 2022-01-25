//! The "kernel" is the interface from the interpreter to the outside world.

use dada_ir::{filename::Filename, function::Function};
use parking_lot::Mutex;
use salsa::DebugWithDb;

use crate::{heap_graph::HeapGraph, value::Value};

#[async_trait::async_trait]
pub trait Kernel: Send + Sync {
    /// Implementation for the `print` intrinsic, that prints a line of text.
    async fn print(&self, text: &str) -> eyre::Result<()>;

    /// Prints a newline.
    async fn print_newline(&self) -> eyre::Result<()> {
        self.print("\n").await
    }

    /// Indicates that we have reached the end of a breakpoint expression.
    fn breakpoint_end(
        &self,
        db: &dyn crate::Db,
        breakpoint_filename: Filename,
        breakpoint_index: usize,
        generate_heap_graph: &dyn Fn() -> HeapGraph,
    ) -> eyre::Result<()>;
}

#[derive(Default)]
pub struct BufferKernel {
    buffer: Mutex<String>,
    stop_at_breakpoint: bool,
    breakpoint_callback: Option<BreakpointCallback>,
    heap_graphs: Mutex<Vec<HeapGraph>>,
}

type BreakpointCallback = Box<dyn Fn(&dyn crate::Db, &BufferKernel, &HeapGraph) + Send + Sync>;

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

    /// Builder method: invoke the given callback instead of accumulating the
    /// heap graph.
    pub fn breakpoint_callback(
        self,
        callback: impl Fn(&dyn crate::Db, &Self, &HeapGraph) + Send + Sync + 'static,
    ) -> Self {
        Self {
            breakpoint_callback: Some(Box::new(callback)),
            ..self
        }
    }

    pub async fn interpret(
        &self,
        db: &dyn crate::Db,
        function: Function,
        arguments: Vec<Value>,
    ) -> eyre::Result<()> {
        crate::interpret(function, db, self, arguments).await
    }

    pub async fn interpret_and_buffer(
        &self,
        db: &dyn crate::Db,
        function: Function,
        arguments: Vec<Value>,
    ) {
        match crate::interpret(function, db, self, arguments).await {
            Ok(()) => {}
            Err(e) => {
                self.append(&e.to_string());
            }
        }
    }

    /// Take the heap graphs from the mutex
    pub fn take_heap_graphs(&mut self) -> Vec<HeapGraph> {
        std::mem::take(self.heap_graphs.get_mut())
    }

    /// Convert the buffer into the output
    pub fn take_buffer(&mut self) -> String {
        std::mem::take(self.buffer.get_mut())
    }

    /// Append text into the output buffer
    pub fn append(&self, s: &str) {
        self.buffer.lock().push_str(s);
    }
}

#[derive(thiserror::Error, Debug)]
#[error("breakpoint expression encountered")]
pub struct BreakpointExpressionEncountered;

#[async_trait::async_trait]
impl Kernel for BufferKernel {
    async fn print(&self, message: &str) -> eyre::Result<()> {
        self.append(message);
        Ok(())
    }

    fn breakpoint_end(
        &self,
        db: &dyn crate::Db,
        filename: Filename,
        index: usize,
        generate_heap_graph: &dyn Fn() -> HeapGraph,
    ) -> eyre::Result<()> {
        tracing::debug!(
            "breakpoint_end(filename={:?}, index={:?})",
            filename.debug(db),
            index
        );

        let heap_graph = generate_heap_graph();

        if let Some(cb) = &self.breakpoint_callback {
            cb(db, self, &heap_graph);
        } else {
            self.heap_graphs.lock().push(heap_graph);
        }

        if self.stop_at_breakpoint {
            return Err(BreakpointExpressionEncountered.into());
        }

        Ok(())
    }
}
