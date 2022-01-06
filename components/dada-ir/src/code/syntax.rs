use crate::{
    op::Op,
    span::Span,
    storage_mode::StorageMode,
    word::{SpannedOptionalWord, Word},
};
use dada_id::{id, tables};
use salsa::DebugWithDb;

use super::Code;

salsa::entity2! {
    entity Tree in crate::Jar {
        origin: Code,
        #[value ref] data: TreeData,
        spans: Spans,
    }
}

impl<Db: ?Sized + crate::Db> DebugWithDb<Db> for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        f.debug_struct("syntax::Tree")
            .field("origin", &self.origin(db.as_dyn_ir_db()).debug(db)) // FIXME
            .finish()
    }
}

/// Stores the ast for a function.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeData {
    /// Interning tables for expressions and the like.
    pub tables: Tables,

    /// The root
    pub root_expr: Expr,
}

impl<Db: ?Sized + crate::Db> DebugWithDb<Db> for TreeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, _db: &Db) -> std::fmt::Result {
        f.debug_struct("syntax::Tree")
            .field("root_expr", &self.root_expr) // FIXME
            .finish()
    }
}

tables! {
    /// Tables that store the data for expr in the AST.
    /// You can use `tables[expr]` (etc) to access the data.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Tables {
        exprs: alloc Expr => ExprData,
        named_exprs: alloc NamedExpr => NamedExprData,
        local_variable_decls: alloc LocalVariableDecl => LocalVariableDeclData,
    }
}

origin_table! {
    /// Side table that contains the spans for everything in a syntax tree.
    /// This isn't normally needed except for diagnostics, so it's
    /// kept separate to avoid reducing incremental reuse.
    /// You can request it by invoking the `spans`
    /// method in the `dada_parse` prelude.
    #[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
    pub struct Spans {
        expr_spans: Expr => Span,
        named_expr_spans: NamedExpr => Span,
        local_variable_decl_spans: LocalVariableDecl => LocalVariableDeclSpan,
    }
}

id!(pub struct Expr);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub enum ExprData {
    Id(Word),

    /// true, false
    BooleanLiteral(bool),

    /// `22`, `22_222`, etc
    IntegerLiteral(Word),

    /// `"foo"` with no format strings
    ///
    /// FIXME: We should replace the FormatString token with a Concatenate
    /// that has parsed expressions.
    StringLiteral(Word),

    /// `expr.ident`
    Dot(Expr, Word),

    /// `expr.await`
    Await(Expr),

    /// `expr(id: expr, ...)`
    Call(Expr, Vec<NamedExpr>),

    /// `expr.share`
    Share(Expr),

    /// `expr.lease`
    Lease(Expr),

    /// `expr.give`
    Give(Expr),

    /// `[shared|var|atomic] x = expr`
    Var(LocalVariableDecl, Expr),

    /// `expr`
    Parenthesized(Expr),

    /// `(expr)` of len != 1
    Tuple(Vec<Expr>),

    /// `if condition { block } [else { block }]`
    If(Expr, Expr, Option<Expr>),

    /// `atomic { block }`
    Atomic(Expr),

    /// `loop { block }`
    Loop(Expr),

    /// `while condition { block }`
    While(Expr, Expr),

    // `{ ... }`, but only as part of a control-flow construct
    Seq(Vec<Expr>),

    /// `a + b`
    Op(Expr, Op, Expr),

    /// `a += b`
    OpEq(Expr, Op, Expr),

    /// `a := b`
    Assign(Expr, Expr),

    /// parse or other error
    Error,
}

id!(pub struct LocalVariableDecl);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub struct LocalVariableDeclData {
    pub mode: Option<StorageMode>,
    pub name: Word,
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct LocalVariableDeclSpan {
    pub mode_span: Span,
    pub name_span: Span,
}

id!(pub struct NamedExpr);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub struct NamedExprData {
    pub name: SpannedOptionalWord,
    pub expr: Expr,
}
