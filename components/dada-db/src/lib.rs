use dada_brew::prelude::*;
use dada_ir::{
    code::bir::Bir,
    diagnostic::Diagnostic,
    input_file::InputFile,
    item::Item,
    span::{FileSpan, LineColumn, Offset},
    word::{ToString, Word},
};
use dada_parse::prelude::*;
use dada_validate::prelude::*;
use salsa::DebugWithDb;

#[salsa::db(
    dada_breakpoint::Jar,
    dada_brew::Jar,
    dada_check::Jar,
    dada_error_format::Jar,
    dada_execute::Jar,
    dada_ir::Jar,
    dada_lex::Jar,
    dada_parse::Jar,
    dada_validate::Jar
)]
#[derive(Default)]
pub struct Db {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for Db {}

impl salsa::ParallelDatabase for Db {
    fn snapshot(&self) -> salsa::Snapshot<Self> {
        salsa::Snapshot::new(Db {
            storage: self.storage.snapshot(),
        })
    }
}

impl Db {
    pub fn new_input_file(&mut self, name: impl ToString, source_text: String) -> InputFile {
        let name = Word::intern(self, name);
        InputFile::new(self, name, source_text, vec![])
    }

    /// Set the breakpoints within the given file where the interpreter stops and executes callbacks.
    pub fn set_breakpoints(&mut self, input_file: InputFile, locations: Vec<LineColumn>) {
        input_file.set_breakpoint_locations(self).to(locations);
    }

    /// Checks `input_file` for compilation errors and returns all relevant diagnostics.
    pub fn diagnostics(&self, input_file: InputFile) -> Vec<Diagnostic> {
        dada_check::check_input_file::accumulated::<dada_ir::diagnostic::Diagnostics>(
            self, input_file,
        )
    }

    /// Checks `input_file` for a function with the given name
    pub fn main_function(&self, input_file: InputFile) -> Option<Bir> {
        let source_file = input_file.source_file(self);

        // If the user included top-level expressions, brew those.
        if let Some(main_fn) = source_file.main_fn(self) {
            return Some(main_fn.brew(self));
        }

        // Otherwise, search for a function named `main`.
        let name = Word::intern(self, "main");
        for item in input_file.items(self) {
            if let Item::Function(function) = item {
                let function_name = function.name(self);
                if name == function_name {
                    return Some(function.brew(self));
                }
            }
        }

        None
    }

    /// Parses `input_file` and returns a list of the items within.
    pub fn items(&self, input_file: InputFile) -> Vec<Item> {
        input_file.items(self).clone()
    }

    /// Parses `input_file` and returns a list of the items within.
    pub fn debug_syntax_tree(&self, item: Item) -> Option<impl std::fmt::Debug + '_> {
        Some(item.syntax_tree(self)?.into_debug(self))
    }

    /// Returns the validated tree for `item`.
    pub fn debug_validated_tree(&self, item: Item) -> Option<impl std::fmt::Debug + '_> {
        Some(item.validated_tree(self)?.into_debug(self))
    }

    /// Returns the validated tree for `item`.
    pub fn debug_bir(&self, item: Item) -> Option<impl std::fmt::Debug + '_> {
        Some(item.maybe_brew(self)?.into_debug(self))
    }

    /// Converts a given offset in a given file into line/column information.
    pub fn line_column(&self, input_file: InputFile, offset: Offset) -> LineColumn {
        dada_ir::lines::line_column(self, input_file, offset)
    }

    /// Converts a `FileSpan` into its constituent parts.
    pub fn line_columns(&self, span: FileSpan) -> (InputFile, LineColumn, LineColumn) {
        let start = self.line_column(span.input_file, span.start);
        let end = self.line_column(span.input_file, span.end);
        (span.input_file, start, end)
    }
}
