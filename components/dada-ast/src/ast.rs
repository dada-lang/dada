use crate::{storage_mode::StorageMode, word::Word};
use dada_intern::intern_id;

intern_id!(pub struct Expr);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub enum ExprData {
    Id(Word),
    Await(Expr),
    Call(Expr, Vec<NamedExpr>),
    Share(Expr),
    Give(Expr),
    Var(StorageMode, Word, Expr),
    Block(Block),
    Op(Expr, Op, Expr),
    OpEq(Path, Op, Expr),
    Assign(Path, Expr),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash, Debug)]
pub enum Op {
    Add,
    Subtract,
    Multiply,
    Divide,
    ShiftLeft,
    ShiftRight,
}

intern_id!(pub struct NamedExpr);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub struct NamedExprData {
    name: Word,
    expr: Expr,
}

intern_id!(pub struct Block);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub struct BlockData {
    exprs: Vec<Expr>,
}

intern_id!(pub struct Path);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub struct PathData {
    base: Word,
    fields: Vec<Word>,
}
