use salsa::DebugWithDb;

use crate::word::Word;

#[salsa::input]
pub struct InputFile {
    name: Word,

    #[return_ref]
    source_text: String,
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
