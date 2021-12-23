use crate::word::Word;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Filename(Word);

impl Filename {
    pub fn from(db: &dyn crate::Db, string: impl crate::word::ToString) -> Self {
        Filename(Word::from(db, string))
    }

    pub fn new(word: Word) -> Self {
        Filename(word)
    }

    pub fn as_str(self, db: &dyn crate::Db) -> &str {
        self.0.as_str(db)
    }
}
