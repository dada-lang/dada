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
    // 2-character ops (must come first!)
    PlusEqual => "+=",
    MinusEqual => "-=",
    TimesEqual => "*=",
    DividedByEqual => "/=",
    ColonEqual => ":=",
    EqualEqual => "==",
    GreaterEqual => ">=",
    LessEqual => "<=",
    RightArrow => "->",

    // 1-character ops
    Plus => "+",
    Minus => "-",
    Times => "*",
    DividedBy => "/",
    Colon => ":",
    SemiColon => ";",
    Equal => "=",
    LessThan => "<",
    GreaterThan => ">",
    LeftAngle => "<",
    RightAngle => ">",
    Dot => ".",
}

#[derive(Debug, PartialEq, Eq)]
pub struct BinaryOp {
    pub binary_op: Op,
    pub assign_op: Op,
}

impl Op {
    /// Operators that can be adjacent to other operators. e.g. we can have `<<`
    /// and parse that as two left angle brackets. Note that we *cannot* parse
    /// that as two "less than" operators.
    pub const ACCEPT_ADJACENT: &'static [Op] = &[Op::LeftAngle, Op::RightAngle];
}

/// Returns a table mapping binary operators like `+` to their `+=` form.
#[salsa::memoized(in crate::Jar ref)]
#[allow(clippy::needless_lifetimes)]
pub fn binary_ops(_db: &dyn crate::Db) -> Vec<BinaryOp> {
    vec![
        BinaryOp {
            binary_op: Op::Plus,
            assign_op: Op::PlusEqual,
        },
        BinaryOp {
            binary_op: Op::Minus,
            assign_op: Op::MinusEqual,
        },
        BinaryOp {
            binary_op: Op::Times,
            assign_op: Op::TimesEqual,
        },
        BinaryOp {
            binary_op: Op::DividedBy,
            assign_op: Op::DividedByEqual,
        },
    ]
}

/// Check that the operator strings have the longest strings first.
#[allow(dead_code)]
const fn check(i: usize, j: usize) {
    if j >= Op::ALL.len() {
        return;
    }
    assert!(Op::STRS[i].len() >= Op::STRS[j].len());
    check(i + 1, j + 1);
}

/// Check that the operator strings have the longest strings first.
const _: () = check(0, 1);
