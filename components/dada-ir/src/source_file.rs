use crate::function::Function;
use crate::{input_file::InputFile, item::Item};

#[salsa::tracked]
/// The result of parsing an input file like `foo.dada`, or the
/// value from the playground. A program is a collection of files.
pub struct SourceFile {
    #[id]
    input_file: InputFile,

    /// The items found in the file.
    #[return_ref]
    items: Vec<Item>,

    /// Top-level "main" function from this file (if any).
    /// This function will also be present in `items`.
    ///
    /// This is not a function declaed with `fn` but rather just
    /// code the user added at the top of the file.
    main_fn: Option<Function>,
}

pub const TOP_LEVEL_FN: &str = "builtin@main";

impl<Db: ?Sized + crate::Db> salsa::DebugWithDb<Db> for SourceFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        let db = db.as_dyn_ir_db();
        f.debug_struct("SourceFile")
            .field("input_file", &self.input_file(db).debug(db))
            .field("items", &self.input_file(db).debug(db))
            .field("main_fn", &self.main_fn(db).debug(db))
            .finish()
    }
}
