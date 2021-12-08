use dada_ast::word::Word;
use dada_collections::Map;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Keyword {
    Class,
    Var,
    Give,
    Share,
    Shared,
    Atomic,
    Fn,
    Async,
}

#[salsa::memoized(in crate::Jar)]
pub(crate) fn keywords(db: &dyn crate::Lexer) -> Map<Word, Keyword> {
    let mut map = Map::default();
    map.insert(Word::from(db, "class"), Keyword::Class);
    map.insert(Word::from(db, "var"), Keyword::Var);
    map.insert(Word::from(db, "give"), Keyword::Give);
    map.insert(Word::from(db, "share"), Keyword::Share);
    map.insert(Word::from(db, "shared"), Keyword::Shared);
    map.insert(Word::from(db, "atomic"), Keyword::Atomic);
    map.insert(Word::from(db, "fn"), Keyword::Fn);
    map.insert(Word::from(db, "async"), Keyword::Async);
    map
}
