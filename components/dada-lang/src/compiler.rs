use dada_ir_ast::{
    ast::{AstFunction, AstItem, AstMember},
    diagnostic::Diagnostic,
    inputs::SourceFile,
};
use dada_util::{Context, Fallible};
use salsa::Database as _;

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

    pub fn fn_asts(&mut self, source_file: SourceFile) -> String {
        use std::fmt::Write;

        let mut output = String::new();

        self.db.attach(|_db| {
            writeln!(
                output,
                "# fn parse tree from {}",
                source_file.path(&self.db)
            )
            .unwrap();
            writeln!(output).unwrap();

            writeln!(output, "{}", fn_asts(&self.db, source_file)).unwrap();
        });

        output
    }
}

#[salsa::tracked]
fn check_all(db: &dyn salsa::Database, source_file: SourceFile) {
    let module = source_file.parse(db);

    for item in module.items(db) {
        match *item {
            AstItem::SourceFile(_source_file) => (),
            AstItem::Use(_use_item) => (),
            AstItem::Class(class_item) => {
                for member in &class_item.members(db) {
                    match member {
                        AstMember::Field(_field_decl) => (),
                        AstMember::Function(function) => check_fn(db, *function),
                    }
                }
            }
            AstItem::Function(function) => {
                check_fn(db, function);
            }
        }
    }
}

fn check_fn<'db>(db: &'db dyn salsa::Database, function: AstFunction<'db>) {
    if let Some(body) = function.body(db) {
        let _block = body.block(db);
    }
}

fn fn_asts(db: &dyn salsa::Database, source_file: SourceFile) -> String {
    use std::fmt::Write;

    let mut output = String::new();

    let module = source_file.parse(db);

    for item in module.items(db) {
        match *item {
            AstItem::SourceFile(_source_file) => (),
            AstItem::Use(_use_item) => (),
            AstItem::Class(class_item) => {
                writeln!(output, "## class `{}`", class_item.name(db)).unwrap();
                for member in &class_item.members(db) {
                    match member {
                        AstMember::Field(_field_decl) => (),
                        AstMember::Function(function) => {
                            writeln!(output, "### fn `{}`", function.name(db).id).unwrap();
                            writeln!(output, "").unwrap();
                            writeln!(output, "{}", fn_asts_fn(db, *function)).unwrap();
                        }
                    }
                }
            }
            AstItem::Function(function) => {
                writeln!(output, "## fn `{}`", function.name(db).id).unwrap();
                writeln!(output, "").unwrap();
                writeln!(output, "{}", fn_asts_fn(db, function)).unwrap();
            }
        }
    }

    return output;

    fn fn_asts_fn<'db>(db: &'db dyn salsa::Database, function: AstFunction<'db>) -> String {
        if let Some(body) = function.body(db) {
            let block = body.block(db);
            format!("{block:#?}")
        } else {
            format!("None")
        }
    }
}
