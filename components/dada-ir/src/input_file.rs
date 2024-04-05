use salsa::DebugWithDb;

use crate::{
    span::{Anchored, LineColumn},
    word::Word,
};

#[salsa::input]
#[customize(DebugWithDb)]
pub struct InputFile {
    pub name: Word,

    /// The raw contents of this input file, as a string.
    #[return_ref]
    pub source_text: String,

    /// The locations of any breakpoints set in this file.
    #[return_ref]
    pub breakpoint_locations: Vec<LineColumn>,
}

impl InputFile {
    pub fn name_str(self, db: &dyn crate::Db) -> &str {
        self.name(db).string(db)
    }
}

impl Anchored for InputFile {
    fn input_file(&self, _db: &dyn crate::Db) -> InputFile {
        *self
    }
}

impl DebugWithDb<dyn crate::Db + '_> for InputFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        f.debug_tuple("SourceFile")
            .field(&self.name(db).debug(db))
            .finish()
    }
}
