use dada_error_format::format_diagnostics;
use dada_execute::kernel::BufferKernel;
use dada_ir::{filename::Filename, span::LineColumn};
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
            let mut kernel = BufferKernel::new();
            kernel.interpret_and_buffer(&db, function, vec![]).await;
            kernel.take_buffer()
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

/// Execute the dada code up until the (0-based) line/column.
#[wasm_bindgen]
pub async fn execute_until(code: String, line0: u32, column0: u32) -> ExecutionResult {
    let mut db = dada_db::Db::default();
    let filename = Filename::from(&db, "input.dada");
    db.update_file(filename, code);

    let diagnostics = db.diagnostics(filename);

    let breakpoint =
        dada_breakpoint::breakpoint::find(&db, filename, LineColumn::new0(line0, column0));
    let mut kernel = BufferKernel::new()
        .breakpoint(breakpoint)
        .stop_at_breakpoint(true);
    match db.function_named(filename, "main") {
        Some(function) => {
            kernel.interpret_and_buffer(&db, function, vec![]).await;
        }
        None => {
            kernel.append(&format!("no `main` function in `{}`", filename.as_str(&db)));
        }
    };

    let output = kernel.take_buffer();
    let heap_graphs = kernel.take_heap_graphs();

    let diagnostics = if diagnostics.is_empty() {
        String::new()
    } else {
        format_diagnostics(&db, &diagnostics).unwrap()
    };

    let heap_capture: String = heap_graphs
        .into_iter()
        .map(|hg| hg.graphviz(&db, false))
        .collect();

    ExecutionResult {
        diagnostics,
        output,
        heap_capture,
    }
}
