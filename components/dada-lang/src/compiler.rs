use dada_ir_ast::{diagnostic::Diagnostic, inputs::SourceFile};
use dada_util::{Context, Fallible};

use crate::db::Database;

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
        check_parse::accumulated::<Diagnostic>(&self.db, source_file)
    }
}

#[salsa::tracked]
fn check_parse(db: &dyn salsa::Database, source_file: SourceFile) {
    source_file.parse(db);
}
