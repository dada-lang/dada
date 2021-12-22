use crate::word::Word;
use dada_collections::Map;

macro_rules! define_keywords {
    (
        $(
            $name:ident => $str:expr,
        )*
    ) => {
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        pub enum Keyword {
            $($name,)*
        }

        impl Keyword {
            const ALL: &'static [Keyword] = &[
                $(Keyword::$name,)*
            ];

            const STRS: &'static [&'static str] = &[
                $($str,)*
            ];

            pub fn all() -> impl Iterator<Item = Keyword> {
                Self::ALL.iter().copied()
            }

            pub fn str(self) -> &'static str {
                Self::STRS[self as usize]
            }

            pub fn word(self, db: &dyn crate::Db) -> Word {
                Word::from(db, self.str())
            }
        }
    }
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`{}`", self.str())
    }
}

define_keywords! {
    Class => "class",
    Var => "var",
    Give => "give",
    Share => "share",
    Shared => "shared",
    Lease => "lease",
    Leased => "leased",
    Atomic => "atomic",
    Fn => "fn",
    Async => "async",
    Await => "await",
    If => "if",
    Else => "else",
    Loop => "loop",
    True => "true",
    False => "false",
    While => "while",
}

#[salsa::memoized(in crate::Jar ref)]
pub fn keywords(db: &dyn crate::Db) -> Map<Word, Keyword> {
    Keyword::all().map(|kw| (kw.word(db), kw)).collect()
}
