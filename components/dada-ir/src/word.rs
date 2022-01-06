use crate::span::FileSpan;

use super::{Db, Jar};

#[salsa::interned(Word in Jar)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WordData {
    pub string: String,
}

impl Word {
    pub fn from<DB: ?Sized + Db>(db: &DB, string: impl ToString) -> Self {
        WordData {
            string: string.to_string(),
        }
        .intern(db)
    }

    pub fn as_str<DB: ?Sized + Db>(self, db: &DB) -> &str {
        &self.data(db).string
    }

    pub fn len(self, db: &dyn crate::Db) -> u32 {
        self.as_str(db).len() as u32
    }
}

impl<Db: ?Sized + crate::Db> salsa::DebugWithDb<Db> for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_str(db), f)
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

salsa::entity2! {
    /// A "spanned word" is a `Word` that also carries a span. Useful for things like
    /// argument names etc where we want to carry the span through many phases
    /// of compilation.
    entity SpannedWord in crate::Jar {
        #[id] word: Word,
        span: FileSpan,
    }
}

impl SpannedWord {
    pub fn as_str(self, db: &dyn crate::Db) -> &str {
        self.word(db).as_str(db)
    }
}

salsa::entity2! {
    /// An optional SpannedOptionalWord is an identifier that may not be persent; it still carries
    /// a span for where the label *would have gone* had it been present (as compared to
    /// an `Option<Label>`).
    entity SpannedOptionalWord in crate::Jar {
        #[id] word: Option<Word>,
        span: FileSpan,
    }
}

impl SpannedOptionalWord {
    pub fn as_str(self, db: &dyn crate::Db) -> Option<&str> {
        Some(self.word(db)?.as_str(db))
    }
}
