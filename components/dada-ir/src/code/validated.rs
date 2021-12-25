//! The "validated" IR is the IR that we use for type checks
//! and so forth. It is still in tree form and is mildly
//! desugared and easy to work with.

use crate::{class::Class, func::Function, op::Op, storage_mode::StorageMode, word::Word};
use dada_id::{id, prelude::*, tables};
use salsa::DebugWithDb;

use super::syntax;

/// Stores the ast for a function.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tree {
    /// Interning tables for expressions and the like.
    tables: Tables,

    /// The root
    root_expr: Expr,
}

impl DebugWithDb<dyn crate::Db + '_> for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, _db: &dyn crate::Db) -> std::fmt::Result {
        f.debug_struct("validated::Tree")
            .field("root_expr", &self.root_expr) // FIXME
            .finish()
    }
}

impl Tree {
    pub fn new(tables: Tables, root_expr: Expr) -> Self {
        Self { tables, root_expr }
    }

    pub fn tables(&self) -> &Tables {
        &self.tables
    }

    pub fn root_expr(&self) -> Expr {
        self.root_expr
    }

    pub fn max_local_variable(&self) -> LocalVariable {
        LocalVariable::max_key(&self.tables)
    }
}

tables! {
    /// Tables that store the data for expr in the AST.
    /// You can use `tables[expr]` (etc) to access the data.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Tables {
        local_variables: alloc LocalVariable => LocalVariableData,
        exprs: alloc Expr => ExprData,
        named_exprs: alloc NamedExpr => NamedExprData,
        places: alloc Place => PlaceData,
    }
}

origin_table! {
    /// Side table that contains the spans for everything in a syntax tree.
    /// This isn't normally needed except for diagnostics, so it's
    /// kept separate to avoid reducing incremental reuse.
    /// You can request it by invoking the `spans`
    /// method in the `dada_parse` prelude.
    #[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
    pub struct Origins {
        expr_spans: Expr => syntax::Expr,
        place_spans: Place => syntax::Expr,
        named_exprs: NamedExpr => syntax::NamedExpr,
        local_variables: LocalVariable => syntax::Expr,
    }
}

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

    /// `()` or `(a, b, ...)` (i.e., expr seq cannot have length 1)
    Tuple(Vec<Expr>),

    /// `if condition { block } [else { block }]`
    If(Expr, Expr, Expr),

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

    /// expr[0]; expr[1]; ...
    Seq(Vec<Expr>),

    /// `a + b`
    Op(Expr, Op, Expr),

    /// `a := b`
    Assign(Place, Expr),

    /// parse or other error
    Error,
}

impl DebugWithDb<dyn crate::Db + '_> for ExprData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, _db: &dyn crate::Db) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f) // FIXME
    }
}

id!(pub struct Place);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub enum PlaceData {
    LocalVariable(LocalVariable),
    Function(Function),
    Class(Class),
    Dot(Place, Word),
}

id!(pub struct NamedExpr);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub struct NamedExprData {
    pub name: Word,
    pub expr: Expr,
}
