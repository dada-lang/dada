use crate::{
    class::Class, func::Function, op::Op, span::Span, span_table::EntireSpan,
    storage_mode::StorageMode, word::Word,
};
use dada_id::{id, tables};

use super::syntax::{self, NamedExprSpan};

/// Stores the ast for a function.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tree {
    /// Interning tables for expressions and the like.
    pub tables: Tables,

    /// The root
    pub root_expr: Expr,
}

tables! {
    /// Tables that store the data for expr in the AST.
    /// You can use `tables[expr]` (etc) to access the data.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Tables {
        local_variables: alloc LocalVariable => LocalVariableData,
        exprs: alloc Expr => ExprData,
        named_exprs: alloc NamedExpr => NamedExprData,
        blocks: alloc Block => BlockData,
    }
}

#[derive(Default)]
pub struct Spans;

id!(pub struct LocalVariable);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub struct LocalVariableData {
    /// Name given to this variable by the user.
    /// If it is None, then this is a temporary
    /// introduced by the compiler.
    pub name: Option<Word>,
    pub storage_mode: StorageMode,
}

id!(pub struct Expr);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub enum ExprData {
    /// Reference to a local variable
    Place(Place),

    /// true, false
    BooleanLiteral(bool),

    /// `22`, `22_222`, etc
    IntegerLiteral(Word),

    /// `"foo"` with no format strings
    StringLiteral(Word),

    /// `expr.await`
    Await(Expr),

    /// `expr(id: expr, ...)`
    Call(Expr, Vec<NamedExpr>),

    /// `expr.share`
    Share(Place),

    /// `expr.lease`
    Lease(Place),

    /// `expr.give`
    Give(Place),

    /// `if condition { block } [else { block }]`
    If(Expr, Expr, Option<Expr>),

    /// `atomic { block }`
    Atomic(Expr),

    /// `loop { block }`
    Loop(Expr),

    /// `break [from expr] [with value]`
    ///
    /// * `from_expr`: Identifies the loop from which we are breaking
    /// * `with_value`: The value produced by the loop
    Break { from_expr: Expr, with_value: Expr },

    /// `continue`
    ///
    /// * `0`: identifies the loop with which we are continuing.
    Continue(Expr),

    /// `break [from expr] [with value]`
    Return(Expr),

    /// `a + b`
    Op(Expr, Op, Expr),

    /// `a := b`
    Assign(Place, Expr),

    /// parse or other error
    Error,
}

id!(pub struct Place);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub enum PlaceData {
    LocalVariable(LocalVariable),
    Function(Function),
    Class(Class),
    Dot(LocalVariable, Word),
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
