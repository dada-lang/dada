use dada_ir_ast::{
    ast::{Function, Item, Member},
    diagnostic::Diagnostic,
    inputs::SourceFile,
};
use dada_util::{Context, Fallible};

use crate::db::Database;
use dada_parser::prelude::*;

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

    pub fn check_all(&mut self, source_file: SourceFile) -> Vec<Diagnostic> {
        check_all::accumulated::<Diagnostic>(&self.db, source_file)
    }
}

#[salsa::tracked]
fn check_all(db: &dyn salsa::Database, source_file: SourceFile) {
    let module = source_file.parse(db);

    for item in module.items(db) {
        match *item {
            Item::SourceFile(_source_file) => (),
            Item::Use(_use_item) => (),
            Item::Class(class_item) => {
                for member in &class_item.members(db) {
                    match member {
                        Member::Field(_field_decl) => (),
                        Member::Function(function) => check_fn(db, *function),
                    }
                }
            }
            Item::Function(function) => {
                check_fn(db, function);
            }
        }
    }
}

fn check_fn<'db>(db: &'db dyn salsa::Database, function: Function<'db>) {
    let _ = function.body_block(db);
}
