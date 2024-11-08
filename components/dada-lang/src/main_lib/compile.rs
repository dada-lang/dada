use std::path::Path;

use dada_compiler::{Compiler, RealFs};
use dada_ir_ast::diagnostic::Level;
use dada_util::{bail, Fallible};

use crate::CompileOptions;

use super::Main;

impl Main {
    pub(super) fn compile(&mut self, compile_options: &CompileOptions) -> Fallible<()> {
        let mut compiler = Compiler::new(RealFs);
        let source_url = RealFs::url(Path::new(&compile_options.input))?;
        let source_file = compiler.load_source_file(&source_url)?;
        let diagnostics = compiler.check_all(source_file);

        for diagnostic in &diagnostics {
            eprintln!(
                "{}",
                diagnostic.render(&compiler, &self.global_options.render_opts())
            );
        }

        if diagnostics.iter().any(|d| d.level >= Level::Error) {
            bail!("compilation failed due to errors");
        }

        Ok(())
    }
}
