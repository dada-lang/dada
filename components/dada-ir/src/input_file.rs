use salsa::DebugWithDb;

use crate::{span::LineColumn, word::Word};

#[salsa::input]
pub struct InputFile {
    name: Word,

    /// The raw contents of this input file, as a string.
    #[return_ref]
    source_text: String,

    /// The locations of any breakpoints set in this file.
    #[return_ref]
    breakpoint_locations: Vec<LineColumn>,
}

impl InputFile {
    pub fn name_str(self, db: &dyn crate::Db) -> &str {
        self.name(db).string(db)
    }
}

impl DebugWithDb<dyn crate::Db + '_> for InputFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        f.debug_tuple("SourceFile")
            .field(&self.name(db).debug(db))
            .finish()
    }
}
