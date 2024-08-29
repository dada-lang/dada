use dada_util::Fallible;

use crate::{compiler::Compiler, error_reporting::RenderDiagnostic, CompileOptions};

use super::Main;

impl Main {
    pub(super) fn compile(&mut self, compile_options: &CompileOptions) -> Fallible<()> {
        let mut compiler = Compiler::new();
        let source_file = compiler.load_input(&compile_options.input)?;
        let diagnostics = compiler.parse(source_file);

        for diagnostic in diagnostics {
            diagnostic.render(&self.global_options, compiler.db());
        }

        Ok(())
    }
}
