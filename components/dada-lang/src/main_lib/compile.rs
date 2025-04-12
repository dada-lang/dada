use std::{path::Path, sync::mpsc::Sender};

use dada_compiler::{Compiler, RealFs};
use dada_ir_ast::{DebugEvent, diagnostic::Level};
use dada_util::{Fallible, bail};

use crate::CompileOptions;

use super::Main;

impl Main {
    pub(super) fn compile(
        &mut self,
        compile_options: &CompileOptions,
        debug_tx: Option<Sender<DebugEvent>>,
    ) -> Fallible<()> {
        let debug_mode = debug_tx.is_some();
        let mut compiler = Compiler::new(RealFs::default(), debug_tx);
        let source_url = Path::new(&compile_options.input);
        let source_file = compiler.load_source_file(source_url)?;
        let bytes = if compile_options.emit_wasm.is_some() {
            compiler.codegen_main_fn(source_file)
        } else {
            &None
        };
        let diagnostics = compiler.check_all(source_file);

        for diagnostic in &diagnostics {
            eprintln!(
                "{}",
                diagnostic.render(&compiler, &self.global_options.render_opts())
            );
        }

        // In debug mode, diagnostics get reported to the `debug_tx` and aren't considered errors.
        if !debug_mode && diagnostics.iter().any(|d| d.level >= Level::Error) {
            bail!("compilation failed due to errors");
        }

        if let Some(wasm_path) = compile_options.emit_wasm.as_ref() {
            let wasm_path = Path::new(wasm_path);
            if let Some(bytes) = bytes {
                std::fs::write(wasm_path, bytes)?;
            }
        }

        Ok(())
    }
}
