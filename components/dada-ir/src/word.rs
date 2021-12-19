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
