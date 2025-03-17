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

        Ok(())
    }
}
