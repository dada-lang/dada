use crate::{storage_mode::StorageMode, token_tree::TokenTree, word::Word};
use dada_id::{id, tables};

salsa::entity2! {
    entity Code in crate::Jar {
        tokens: TokenTree,
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
    If(Expr, Block, Option<Block>),

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
    pub name: Word,
    pub expr: Expr,
}

id!(pub struct Block);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub struct BlockData {
    pub exprs: Vec<Expr>,
}

id!(pub struct Path);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub struct PathData {
    pub base: Word,
    pub fields: Vec<Word>,
}
