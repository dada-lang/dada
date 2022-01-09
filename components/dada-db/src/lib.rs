use dada_brew::prelude::MaybeBrewExt;
use dada_ir::{
    diagnostic::Diagnostic,
    filename::Filename,
    function::Function,
    item::Item,
    span::{FileSpan, LineColumn, Offset},
    word::Word,
};
use dada_parse::prelude::*;
use dada_validate::prelude::*;
use salsa::DebugWithDb;

#[salsa::db(
    dada_brew::Jar,
    dada_check::Jar,
    dada_error_format::Jar,
    dada_execute::Jar,
    dada_ir::Jar,
    dada_lex::Jar,
    dada_manifest::Jar,
    dada_parse::Jar,
    dada_validate::Jar
)]
#[derive(Default)]
pub struct Db {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for Db {
    fn salsa_runtime(&self) -> &salsa::Runtime {
        self.storage.runtime()
    }
}

impl salsa::ParallelDatabase for Db {
    fn snapshot(&self) -> salsa::Snapshot<Self> {
        salsa::Snapshot::new(Db {
            storage: self.storage.snapshot(),
        })
    }
}

impl Db {
    pub fn update_file(&mut self, filename: Filename, source_text: String) {
        dada_manifest::source_text::set(self, filename, source_text)
    }

    /// Checks `filename` for compilation errors and returns all relevant diagnostics.
    pub fn diagnostics(&self, filename: Filename) -> Vec<Diagnostic> {
        dada_check::check_filename::accumulated::<dada_ir::diagnostic::Diagnostics>(self, filename)
    }

    /// Checks `filename` for a "main" function
    pub fn function_named(&self, filename: Filename, name: &str) -> Option<Function> {
        let name = Word::from(self, name);
        for item in filename.items(self) {
            if let Item::Function(function) = item {
                let function_name = function.name(self);
                if name == function_name {
                    return Some(*function);
                }
            }
        }
        None
    }

    /// Parses `filename` and returns a lits of the items within.
    pub fn items(&self, filename: Filename) -> Vec<Item> {
        filename.items(self).clone()
    }

    /// Parses `filename` and returns a lits of the items within.
    pub fn debug_syntax_tree(&self, item: Item) -> Option<impl std::fmt::Debug + '_> {
        Some(item.syntax_tree(self)?.into_debug(self))
    }

    /// Returns the validated tree for `item`.
    pub fn debug_validated_tree(&self, item: Item) -> Option<impl std::fmt::Debug + '_> {
        Some(item.validated_tree(self)?.data(self).into_debug(self))
    }

    /// Returns the validated tree for `item`.
    pub fn debug_bir(&self, item: Item) -> Option<impl std::fmt::Debug + '_> {
        Some(item.maybe_brew(self)?.data(self).into_debug(self))
    }

    /// Converts a given offset in a given file into line/column information.
    pub fn line_column(&self, filename: Filename, offset: Offset) -> LineColumn {
        dada_lex::line_column(self, filename, offset)
    }

    /// Converts a `FileSpan` into its constituent parts.
    pub fn line_columns(&self, span: FileSpan) -> (Filename, LineColumn, LineColumn) {
        let start = self.line_column(span.filename, span.start);
        let end = self.line_column(span.filename, span.end);
        (span.filename, start, end)
    }
}
