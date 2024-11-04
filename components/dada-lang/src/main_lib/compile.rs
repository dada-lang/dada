use std::path::Path;

use dada_ir_ast::diagnostic::Level;
use dada_util::{bail, Fallible};

use crate::{compiler::Compiler, CompileOptions};

use super::Main;

impl Main {
    pub(super) fn compile(&mut self, compile_options: &CompileOptions) -> Fallible<()> {
        let mut compiler = Compiler::new();
        let source_file = compiler.load_input(Path::new(&compile_options.input))?;
        let diagnostics = compiler.check_all(source_file);

        for diagnostic in &diagnostics {
            eprintln!(
                "{}",
                diagnostic.render(compiler.db(), &self.global_options.render_opts())
            );
        }

        if diagnostics.iter().any(|d| d.level >= Level::Error) {
            bail!("compilation failed due to errors");
        }

        Ok(())
    }
}
