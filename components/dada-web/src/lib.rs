#![allow(clippy::unused_unit)] // wasm-bindgen seems to trigger this

use dada_error_format::format_diagnostics;
use dada_execute::kernel::BufferKernel;
use dada_ir::{filename::Filename, span::LineColumn};
use tracing_wasm::WASMLayerConfigBuilder;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    tracing_wasm::set_as_global_default_with_config(
        WASMLayerConfigBuilder::new()
            .set_max_level(tracing::Level::INFO)
            .build(),
    );

    Ok(())
}

#[wasm_bindgen]
#[derive(Default)]
pub struct DadaCompiler {
    db: dada_db::Db,

    /// Current diagnostics emitted by the compiler, formatted
    /// as a string.
    diagnostics: String,

    /// Current output emitted by the program.
    output: String,

    /// If a breakpoint was set, contains graphviz source
    /// for the heap at that point (else empty).
    heap_capture: String,
}

#[wasm_bindgen]
pub fn compiler() -> DadaCompiler {
    Default::default()
}

#[wasm_bindgen]
impl DadaCompiler {
    fn filename(&self) -> Filename {
        Filename::from(&self.db, "input.dada")
    }

    #[wasm_bindgen]
    pub fn with_source_text(mut self, source: String) -> Self {
        // FIXME: reset the database for now
        tracing::debug!("with_source_text: {source:?}");
        self.db = Default::default();
        self.db.update_file(self.filename(), source);
        self
    }

    #[wasm_bindgen]
    pub fn with_breakpoint(mut self, line0: u32, column0: u32) -> Self {
        self.db
            .set_breakpoints(self.filename(), vec![LineColumn::new0(line0, column0)]);
        self
    }

    #[wasm_bindgen]
    pub fn without_breakpoint(mut self) -> Self {
        self.db.set_breakpoints(self.filename(), vec![]);
        self
    }

    #[wasm_bindgen]
    pub async fn execute(mut self) -> Self {
        let filename = self.filename();
        let diagnostics = self.db.diagnostics(filename);

        let mut kernel = BufferKernel::new().stop_at_breakpoint(true);
        match self.db.function_named(filename, "main") {
            Some(function) => {
                kernel
                    .interpret_and_buffer(&self.db, function, vec![])
                    .await;
            }
            None => {
                kernel.append(&format!(
                    "no `main` function in `{}`",
                    filename.as_str(&self.db)
                ));
            }
        };

        self.output = kernel.take_buffer();
        let heap_graphs = kernel.take_heap_graphs();

        tracing::info!(
            "Execution complete: \
            {} bytes of output, \
            {} heaps captured, \
            {} diagnostics.",
            self.output.len(),
            heap_graphs.len(),
            diagnostics.len(),
        );

        self.diagnostics = if diagnostics.is_empty() {
            String::new()
        } else {
            format_diagnostics(&self.db, &diagnostics).unwrap()
        };

        self.heap_capture = heap_graphs
            .into_iter()
            .map(|hg| hg.graphviz(&self.db, false))
            .collect();

        self
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
    pub fn heap(&self) -> String {
        self.heap_capture.clone()
    }
}
