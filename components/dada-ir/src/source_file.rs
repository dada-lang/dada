use crate::code::syntax::Tree;
use crate::{filename::Filename, item::Item};

salsa::entity2! {
    /// The result of parsing an input file like `foo.dada`, or the
    /// value from the playground. A program is a collection of files.
    entity SourceFile in crate::Jar {
        #[id] filename: Filename,

        /// The items found in the file.
        #[value ref] items: Vec<Item>,

        /// Top-level expressions from this file (if any).
        syntax_tree: Option<Tree>,
    }
}

impl<Db: ?Sized + crate::Db> salsa::DebugWithDb<Db> for SourceFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        let db = db.as_dyn_ir_db();
        f.debug_struct("SourceFile")
            .field("filename", &self.filename(db).debug(db))
            .field("items", &self.filename(db).debug(db))
            .finish()
    }
}
