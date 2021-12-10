use crate::{storage_mode::StorageMode, word::Word};
use dada_id::{id, tables};

salsa::entity2! {
    entity Code in crate::Jar {
        #[no_eq] ast: Ast,
    }
}

#[derive(Clone, Debug)]
pub struct Ast {
    pub tables: CodeTables,
    pub block: Block,
}

tables! {
    pub struct CodeTables {
        exprs: alloc Expr => ExprData,
        named_exprs: alloc NamedExpr => NamedExprData,
        blocks: alloc Block => BlockData,
        paths: alloc Path => PathData,
    }
}

id!(pub struct Expr);

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

id!(pub struct NamedExpr);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub struct NamedExprData {
    name: Word,
    expr: Expr,
}

id!(pub struct Block);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub struct BlockData {
    exprs: Vec<Expr>,
}

id!(pub struct Path);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub struct PathData {
    base: Word,
    fields: Vec<Word>,
}
