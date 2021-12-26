// Intrinsic functions or other items
use crate::word::Word;

macro_rules! intrinsic {
    ($($name:ident => $s:expr,)*) => {
        #[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
        pub enum Intrinsic {
            $($name,)*
        }

        impl Intrinsic {
            pub const ALL: &'static [Intrinsic] = &[
                $(Intrinsic::$name,)*
            ];

            pub fn as_str(self, _db: &dyn crate::Db) -> &'static str {
                match self {
                    $(
                        Intrinsic::$name => $s,
                    )*
                }
            }

            pub fn name(self, db: &dyn $crate::Db) -> Word {
                Word::from(db, self.as_str(db))
            }
        }
    }
}

intrinsic! {
    Print => "print",
}
