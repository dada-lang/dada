use crate::span::FileSpan;

#[salsa::interned]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Word {
    #[return_ref]
    pub string: String,
}

impl Word {
    pub fn intern(db: &dyn crate::Db, string: impl ToString) -> Self {
        Word::new(db, string.to_string())
    }

    pub fn as_str(self, db: &dyn crate::Db) -> &str {
        self.string(db)
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(self, db: &dyn crate::Db) -> u32 {
        self.as_str(db).len() as u32
    }
}

impl salsa::DebugWithDb<dyn crate::Db + '_> for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.string(db), f)
    }
}

pub trait ToString {
    fn to_string(self) -> String;
}

impl ToString for String {
    fn to_string(self) -> String {
        self
    }
}

impl ToString for &str {
    fn to_string(self) -> String {
        self.to_owned()
    }
}

impl ToString for &std::path::Path {
    fn to_string(self) -> String {
        self.display().to_string()
    }
}

impl ToString for &std::path::PathBuf {
    fn to_string(self) -> String {
        self.display().to_string()
    }
}

#[salsa::tracked]
/// An optional SpannedOptionalWord is an identifier that may not be persent; it still carries
/// a span for where the label *would have gone* had it been present (as compared to
/// an `Option<Label>`).
pub struct SpannedOptionalWord {
    #[id]
    word: Option<Word>,
    span: FileSpan,
}

impl SpannedOptionalWord {
    pub fn as_str(self, db: &dyn crate::Db) -> Option<&str> {
        Some(self.word(db)?.as_str(db))
    }
}

impl<Db: ?Sized + crate::Db> salsa::DebugWithDb<Db> for SpannedOptionalWord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.as_str(db.as_dyn_ir_db()), f)
    }
}
