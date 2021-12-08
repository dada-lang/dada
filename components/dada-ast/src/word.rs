use super::{Ast, Jar};

#[salsa::interned(Word in Jar)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WordData {
    pub string: String,
}

impl Word {
    pub fn from<DB: ?Sized + Ast>(db: &DB, string: String) -> Self {
        WordData { string }.intern(db)
    }

    pub fn as_str<DB: ?Sized + Ast>(self, db: &DB) -> &str {
        &self.data(db).string
    }
}
