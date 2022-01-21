//! The "kernel" is the interface from the interpreter to the outside world.

use dada_ir::{
    code::{syntax, Code},
    function::Function,
};
use parking_lot::Mutex;

use crate::{execute::StackFrame, heap_graph::HeapGraph, value::Value};

#[async_trait::async_trait]
pub trait Kernel: Send + Sync {
    /// Implementation for the `print` intrinsic, that prints a line of text.
    async fn print(&self, text: &str) -> eyre::Result<()>;

    /// Prints a newline.
    async fn print_newline(&self) -> eyre::Result<()> {
        self.print("\n").await
    }

    /// Indicates that `stack_frame` is on the cusp of completing `expr`.
    /// This gives the kernel a chance to capture the state.
    fn on_cusp(
        &self,
        db: &dyn crate::Db,
        stack_frame: &StackFrame<'_>,
        expr: syntax::Expr,
    ) -> eyre::Result<()>;
}

#[derive(Default)]
pub struct BufferKernel {
    buffer: Mutex<String>,
    breakpoint: Option<(Code, syntax::Expr)>,
    stop_at_breakpoint: bool,
    dump_breakpoint: bool,
    heap_graphs: Mutex<Vec<HeapGraph>>,
}

impl BufferKernel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Buiilder method: if breakpoint is Some, then whenever the breakpoint is
    /// encountered we will capture a heap-graph.
    ///
    /// See the `dada-breakpoint` crate for code to find a breakpoint.
    pub fn breakpoint(self, breakpoint: Option<(Code, syntax::Expr)>) -> Self {
        assert!(self.breakpoint.is_none());
        Self { breakpoint, ..self }
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

    /// Builder method: if `dump_breakpoint` is true, prints graphviz
    /// for heap at each breakpoint into the buffer instead of accumulating
    /// into the internal vector for later inspection.
    pub fn dump_breakpoint(self, dump_breakpoint: bool) -> Self {
        Self {
            dump_breakpoint,
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

    fn on_cusp(
        &self,
        db: &dyn crate::Db,
        stack_frame: &StackFrame<'_>,
        expr: syntax::Expr,
    ) -> eyre::Result<()> {
        if let Some((breakpoint_code, breakpoint_expr)) = self.breakpoint {
            if breakpoint_expr == expr && breakpoint_code == stack_frame.code(db) {
                let heap_graph = HeapGraph::new(db, stack_frame);

                if self.dump_breakpoint {
                    self.append(&heap_graph.graphviz(db, true));
                } else {
                    self.heap_graphs.lock().push(heap_graph);
                }

                if self.stop_at_breakpoint {
                    return Err(BreakpointExpressionEncountered.into());
                }
            }
        }

        Ok(())
    }
}
