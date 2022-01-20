use dada_error_format::format_diagnostics;
use dada_execute::kernel::{BufferKernel, Kernel};
use dada_ir::{
    code::{syntax, Code},
    filename::Filename,
    span::LineColumn,
};
use parking_lot::Mutex;
use thiserror::Error;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ExecutionResult {
    /// Any diagnostics emitted by the compiler, formatted
    /// as a string.
    diagnostics: String,

    /// Output emitted by the program.
    output: String,

    /// If a breakpoint was set, contains graphviz source
    /// for the heap at that point (else empty).
    heap_capture: String,
}

#[wasm_bindgen]
impl ExecutionResult {
    #[wasm_bindgen(getter)]
    #[allow(non_snake_case)]
    pub fn fullOutput(&self) -> String {
        if self.diagnostics.is_empty() {
            self.output.clone()
        } else {
            format!("{}\n{}", self.diagnostics, self.output)
        }
    }

    #[wasm_bindgen(getter)]
    pub fn diagnostics(&self) -> String {
        self.diagnostics.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn output(&self) -> String {
        self.output.clone()
    }

    #[wasm_bindgen(getter)]
    #[allow(non_snake_case)]
    pub fn heapCapture(&self) -> String {
        self.heap_capture.clone()
    }
}

/// Execute the dada code and generate output (plus compiler diagnostics).
#[wasm_bindgen]
pub async fn execute(code: String) -> ExecutionResult {
    let mut db = dada_db::Db::default();
    let filename = Filename::from(&db, "input.dada");
    db.update_file(filename, code);

    let diagnostics = db.diagnostics(filename);

    let output = match db.function_named(filename, "main") {
        Some(function) => {
            let kernel = BufferKernel::new();
            kernel.interpret_and_buffer(&db, function, vec![]).await;
            kernel.into_buffer()
        }
        None => {
            format!("no `main` function in `{}`", filename.as_str(&db))
        }
    };

    let diagnostics = if diagnostics.is_empty() {
        String::new()
    } else {
        format_diagnostics(&db, &diagnostics).unwrap()
    };

    ExecutionResult {
        diagnostics,
        output,
        heap_capture: String::new(),
    }
}

/// Execute the dada code up until the (1-based) line/column.
#[wasm_bindgen]
pub async fn execute_until(code: String, line: u32, column: u32) -> ExecutionResult {
    let mut db = dada_db::Db::default();
    let filename = Filename::from(&db, "input.dada");
    db.update_file(filename, code);

    let diagnostics = db.diagnostics(filename);

    let breakpoint = dada_breakpoint::breakpoint::find(&db, filename, LineColumn { line, column });
    let kernel = WebKernel::new(breakpoint);
    let output = match db.function_named(filename, "main") {
        Some(function) => {
            match dada_execute::interpret(function, &db, &kernel, vec![]).await {
                Ok(()) => {}
                Err(e) => match e.downcast() {
                    Ok(BreakpointExpressionEncountered) => {}
                    Err(e) => kernel.buffer.append(&e.to_string()),
                },
            }
            kernel.buffer.into_buffer()
        }
        None => {
            format!("no `main` function in `{}`", filename.as_str(&db))
        }
    };

    let diagnostics = if diagnostics.is_empty() {
        String::new()
    } else {
        format_diagnostics(&db, &diagnostics).unwrap()
    };

    let heap_capture: String = kernel.heap_graphs.into_inner().into_iter().collect();

    ExecutionResult {
        diagnostics,
        output,
        heap_capture,
    }
}

struct WebKernel {
    buffer: BufferKernel,
    breakpoint: Option<(Code, syntax::Expr)>,
    heap_graphs: Mutex<Vec<String>>,
}

impl WebKernel {
    fn new(breakpoint: Option<(Code, syntax::Expr)>) -> Self {
        Self {
            buffer: BufferKernel::new(),
            breakpoint,
            heap_graphs: Default::default(),
        }
    }
}

#[derive(Error, Debug)]
#[error("breakpoint expression encountered")]
struct BreakpointExpressionEncountered;

#[async_trait::async_trait]
impl Kernel for WebKernel {
    async fn print(&self, text: &str) -> eyre::Result<()> {
        self.buffer.print(text).await
    }

    fn on_cusp(
        &self,
        db: &dyn dada_execute::Db,
        stack_frame: &dada_execute::StackFrame<'_>,
        current_expr: syntax::Expr,
    ) -> eyre::Result<()> {
        match self.breakpoint {
            Some((code, cusp_expr))
                if cusp_expr == current_expr && code == stack_frame.code(db) =>
            {
                let heap_graph = dada_execute::heap_graph::HeapGraph::new(db, stack_frame);
                self.heap_graphs.lock().push(heap_graph.graphviz(db, true));
                Err(BreakpointExpressionEncountered.into())
            }
            _ => Ok(()),
        }
    }
}
