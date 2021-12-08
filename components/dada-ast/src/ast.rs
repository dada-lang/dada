use crate::word::Word;

#[salsa::interned(Expr in crate::Jar)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub enum ExprData {
    Id(Word),
    Await(Expr),
    Call(Expr, Vec<Expr>),
    Share(Expr),
}
