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
    Any => "any",
    Async => "async",
    Atomic => "atomic",
    Await => "await",
    Class => "class",
    Else => "else",
    False => "false",
    Fn => "fn",
    Give => "give",
    If => "if",
    Lease => "lease",
    Leased => "leased",
    Loop => "loop",
    My => "my",
    Return => "return",
    Share => "share",
    Shared => "shared",
    Shlease => "shlease",
    Shleased => "shleased",
    True => "true",
    Our => "our",
    While => "while",
}

#[salsa::memoized(in crate::Jar ref)]
#[allow(clippy::needless_lifetimes)]
pub fn keywords(db: &dyn crate::Db) -> Map<Word, Keyword> {
    Keyword::all().map(|kw| (kw.word(db), kw)).collect()
}
