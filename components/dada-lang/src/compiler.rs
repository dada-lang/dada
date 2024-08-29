use dada_ir_ast::{
    diagnostic::{Diagnostic, Diagnostics},
    inputs::SourceFile,
};
use dada_util::{Context, Fallible};

use crate::{
    db::Database, error_reporting::RenderDiagnostic, Command, CompileOptions, GlobalOptions,
};

pub struct Compiler {
    db: Database,
    source_files: Vec<SourceFile>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            db: Database::default(),
            source_files: vec![],
        }
    }

    pub fn db(&self) -> &Database {
        &self.db
    }

    pub fn load_input(&mut self, input: &str) -> Fallible<SourceFile> {
        let contents = std::fs::read_to_string(input)
            .with_context(|| format!("failed to read input file `{}`", input))?;

        let source_file = SourceFile::new(&self.db, input.to_string(), contents);
        self.source_files.push(source_file);

        Ok(source_file)
    }

    pub fn parse(&mut self, source_file: SourceFile) -> Vec<Diagnostic> {
        dada_ir_ast::parse::SourceFile_parse::accumulated::<Diagnostics>(&self.db, source_file)
    }
}
