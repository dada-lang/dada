use std::path::Path;

use dada_compiler::{Compiler, RealFs};
use dada_util::Fallible;

use crate::RunOptions;

use super::Main;

impl Main {
    pub(super) fn run_command(&mut self, run_options: &RunOptions) -> Fallible<()> {
        let mut compiler = Compiler::new(RealFs::default());
        let source_url = Path::new(&run_options.compile_options.input);
        let source_file = compiler.load_source_file(source_url)?;
        let (bytes, diagnostics) = compiler.codegen_main_fn(source_file);

        for diagnostic in &diagnostics {
            eprintln!(
                "{}",
                diagnostic.render(&compiler, &self.global_options.render_opts())
            );
        }

        let _ = bytes;

        Ok(())
    }
}
