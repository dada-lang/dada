use salsa::DebugWithDb;

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

impl DebugWithDb<dyn crate::Db + '_> for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.string(db), f)
    }
}

#[salsa::interned]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Words {
    #[return_ref]
    pub elements: Vec<Word>,
}

impl Words {
    pub fn from_iter(db: &dyn crate::Db, iter: impl IntoIterator<Item = Word>) -> Words {
        Words::from_vec(db, iter.into_iter().collect())
    }

    pub fn from_vec(db: &dyn crate::Db, mut words: Vec<Word>) -> Words {
        words.shrink_to_fit();
        Words::new(db, words)
    }
}

impl DebugWithDb<dyn crate::Db + '_> for Words {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        f.debug_list()
            .entries(self.elements(db).iter().map(|w| w.debug(db)))
            .finish()
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
