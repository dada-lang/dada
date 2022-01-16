macro_rules! define_operators {
    (
        $(
            $name:ident => $str:expr,
        )*
    ) => {
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        pub enum Op {
            $($name,)*
        }

        impl Op {
            const ALL: &'static [Op] = &[
                $(Op::$name,)*
            ];

            const STRS: &'static [&'static str] = &[
                $($str,)*
            ];

            pub fn all() -> impl Iterator<Item = Op> {
                Self::ALL.iter().copied()
            }

            pub fn str(self) -> &'static str {
                Self::STRS[self as usize]
            }
        }
    }
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.str())
    }
}

define_operators! {
    EqualEqual => "==",
    Plus => "+",
    Minus => "-",
    Times => "*",
    DividedBy => "/",
    LessThan => "<",
    GreaterThan => ">",
}
