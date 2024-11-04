#![feature(trait_upcasting)]

use std::path::Path;

use dada_ir_ast::{
    ast::{AstFunction, AstItem, AstMember},
    diagnostic::Diagnostic,
    inputs::SourceFile,
};
use dada_util::{bail, Fallible};
use salsa::Database as _;

use crate::db::Database;
use dada_parser::prelude::*;

mod db;
pub use crate::db::Db;

pub struct Compiler {
    db: Database,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            db: Database::default(),
        }
    }

    pub fn db(&self) -> &dyn Db {
        &self.db
    }

    /// Add a crate that is rooted in the given `dada` file.
    /// The crate is named after the file name.
    pub fn add_crate_with_root_path(&mut self, root_path: &Path) -> Fallible<()> {
        if root_path.extension().is_none() || root_path.extension().unwrap() != "dada" {
            bail!(
                "crate root path should have `.dada` extension: `{}`",
                root_path.display()
            );
        }

        let Some(crate_name) = root_path.file_stem().unwrap().to_str() else {
            bail!(
                "cannot add crate with non-UTF8 name `{}`",
                root_path.display()
            );
        };

        // For a given crate, the root module would be called
        // `foo.dada` and then any submodules will be in
        // `foo/...`.
        let root_dir_path = root_path.with_extension("");
        if root_dir_path.exists() && !root_dir_path.is_dir() {
            bail!(
                "crate root `{}` requires `{}` to be a directory, not a file",
                root_path.display(),
                root_dir_path.display(),
            );
        }

        self.db.add_crate(crate_name.to_string(), root_dir_path)?;

        Ok(())
    }

    pub fn load_input(&mut self, input: &Path) -> Fallible<SourceFile> {
        self.add_crate_with_root_path(input)?;
        Ok(self.db.source_file(input))
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
                source_file.path(&self.db),
            )
            .unwrap();
            writeln!(output).unwrap();

            writeln!(output, "{}", fn_asts(&self.db, source_file)).unwrap();
        });

        output
    }
}

#[salsa::tracked]
fn check_all(db: &dyn Db, source_file: SourceFile) {
    use dada_check::Check;
    source_file.check(db);
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
                for member in class_item.members(db) {
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
        if let Some(block) = function.body_block(db) {
            format!("{block:#?}")
        } else {
            format!("None")
        }
    }
}
