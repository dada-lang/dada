use dada_ir::{diagnostic::Diagnostic, item::Item, word::Word};
use dada_parse::prelude::*;

#[salsa::db(dada_ir::Jar, dada_lex::Jar, dada_manifest::Jar, dada_parse::Jar)]
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
    pub fn update_file(&mut self, filename: Word, source_text: String) {
        dada_manifest::source_text::set(self, filename, source_text)
    }

    pub fn diagnostics(&self, filename: Word) -> Vec<Diagnostic> {
        let mut all_errors = vec![];

        let (items, errors) = dada_parse::parse_file(self, filename);
        all_errors.extend(errors.iter().cloned());

        for &item in items {
            match item {
                Item::Function(function) => {
                    let (_, errors) = function.parse_ast(self);
                    all_errors.extend(errors.iter().cloned());
                }
                Item::Class(_class) => {}
            }
        }

        all_errors
    }

    pub fn items(&self, filename: Word) -> Vec<Item> {
        let (items, _) = dada_parse::parse_file(self, filename);
        items.clone()
    }
}
