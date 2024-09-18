use dada_ir_ast::diagnostic::Level;
use dada_util::{bail, Fallible};

use crate::{compiler::Compiler, error_reporting::RenderDiagnostic, CompileOptions};

use super::Main;

impl Main {
    pub(super) fn compile(&mut self, compile_options: &CompileOptions) -> Fallible<()> {
        let mut compiler = Compiler::new();
        let source_file = compiler.load_input(&compile_options.input)?;
        let diagnostics = compiler.check_all(source_file);

        for diagnostic in &diagnostics {
            eprintln!("{}", diagnostic.render(&self.global_options, compiler.db()));
        }

        if diagnostics.iter().any(|d| d.level >= Level::Error) {
            bail!("compilation failed due to errors");
        }

        Ok(())
    }
}
