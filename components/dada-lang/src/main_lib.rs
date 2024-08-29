use dada_util::Fallible;

use crate::{
    compiler::Compiler, error_reporting::RenderDiagnostic, Command, CompileOptions, GlobalOptions,
};

mod test;

pub struct Main {
    #[allow(dead_code)]
    global_options: GlobalOptions,
}

impl Main {
    pub fn new(global_options: GlobalOptions) -> Self {
        Self { global_options }
    }

    pub fn run(mut self, command: Command) -> Fallible<()> {
        match command {
            crate::Command::Compile { compile_options } => self.compile(&compile_options)?,
        }
        Ok(())
    }

    pub fn compile(&mut self, compile_options: &CompileOptions) -> Fallible<()> {
        let mut compiler = Compiler::new();
        let source_file = compiler.load_input(&compile_options.input)?;
        let diagnostics = compiler.parse(source_file);

        for diagnostic in diagnostics {
            diagnostic.render(&self.global_options, compiler.db());
        }

        Ok(())
    }
}
