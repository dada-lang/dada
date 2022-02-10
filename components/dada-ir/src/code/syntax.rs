use crate::{
    code::syntax::op::Op,
    in_ir_db::InIrDb,
    in_ir_db::InIrDbExt,
    span::Span,
    storage_mode::StorageMode,
    word::{SpannedOptionalWord, Word},
};
use dada_id::{id, prelude::*, tables};
use salsa::DebugWithDb;

use super::Code;

salsa::entity2! {
    entity Tree in crate::Jar {
        origin: Code,
        #[value ref] data: TreeData,
        #[value ref] spans: Spans,
    }
}

impl DebugWithDb<dyn crate::Db> for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        f.debug_struct("syntax::Tree")
            .field("origin", &self.origin(db).debug(db))
            .field("data", &self.data(db).debug(&self.in_ir_db(db)))
            .finish()
    }
}

impl InIrDb<'_, Tree> {
    fn tables(&self) -> &Tables {
        &self.data(self.db()).tables
    }
}

/// Stores the ast for a function.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeData {
    /// Interning tables for expressions and the like.
    pub tables: Tables,

    /// Parameter declarations
    pub parameter_decls: Vec<LocalVariableDecl>,

    /// The root
    pub root_expr: Expr,
}

impl DebugWithDb<InIrDb<'_, Tree>> for TreeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tree>) -> std::fmt::Result {
        f.debug_struct("syntax::Tree")
            .field("root_expr", &self.root_expr.debug(db)) // FIXME
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

impl DebugWithDb<InIrDb<'_, Tree>> for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tree>) -> std::fmt::Result {
        f.debug_tuple("")
            .field(self)
            .field(&self.data(db.tables()).debug(db))
            .finish()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub enum ExprData {
    Id(Word),

    /// true, false
    BooleanLiteral(bool),

    /// `22`, `22_222`, etc
    IntegerLiteral(Word),

    /// `integer-part.fractional-part`
    FloatLiteral(Word, Word),

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

impl DebugWithDb<InIrDb<'_, Tree>> for ExprData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tree>) -> std::fmt::Result {
        match self {
            ExprData::Id(w) => f.debug_tuple("Id").field(&w.debug(db.db())).finish(),
            ExprData::BooleanLiteral(v) => f.debug_tuple("Boolean").field(&v).finish(),
            ExprData::IntegerLiteral(v) => {
                f.debug_tuple("Integer").field(&v.debug(db.db())).finish()
            }
            ExprData::FloatLiteral(v, d) => f
                .debug_tuple("Float")
                .field(&v.debug(db.db()))
                .field(&d.debug(db.db()))
                .finish(),
            ExprData::StringLiteral(v) => f.debug_tuple("String").field(&v.debug(db.db())).finish(),
            ExprData::Dot(lhs, rhs) => f
                .debug_tuple("Dot")
                .field(&lhs.debug(db))
                .field(&rhs.debug(db.db()))
                .finish(),
            ExprData::Await(e) => f.debug_tuple("Await").field(&e.debug(db)).finish(),
            ExprData::Call(func, args) => f
                .debug_tuple("Call")
                .field(&func.debug(db))
                .field(&args.debug(db))
                .finish(),
            ExprData::Share(e) => f.debug_tuple("Share").field(&e.debug(db)).finish(),
            ExprData::Lease(e) => f.debug_tuple("Lease").field(&e.debug(db)).finish(),
            ExprData::Give(e) => f.debug_tuple("Give").field(&e.debug(db)).finish(),
            ExprData::Var(v, e) => f
                .debug_tuple("Var")
                .field(&v.debug(db))
                .field(&e.debug(db))
                .finish(),
            ExprData::Parenthesized(e) => f.debug_tuple("Share").field(&e.debug(db)).finish(),
            ExprData::Tuple(e) => f.debug_tuple("Tuple").field(&e.debug(db)).finish(),
            ExprData::If(c, t, e) => f
                .debug_tuple("If")
                .field(&c.debug(db))
                .field(&t.debug(db))
                .field(&e.debug(db))
                .finish(),
            ExprData::Atomic(e) => f.debug_tuple("Atomic").field(&e.debug(db)).finish(),
            ExprData::Loop(e) => f.debug_tuple("Loop").field(&e.debug(db)).finish(),
            ExprData::While(c, e) => f
                .debug_tuple("While")
                .field(&c.debug(db))
                .field(&e.debug(db))
                .finish(),
            ExprData::Seq(e) => f.debug_tuple("Seq").field(&e.debug(db)).finish(),
            ExprData::Op(l, o, r) => f
                .debug_tuple("Op")
                .field(&l.debug(db))
                .field(&o)
                .field(&r.debug(db))
                .finish(),
            ExprData::OpEq(l, o, r) => f
                .debug_tuple("OpEq")
                .field(&l.debug(db))
                .field(&o)
                .field(&r.debug(db))
                .finish(),
            ExprData::Assign(l, r) => f
                .debug_tuple("Assign")
                .field(&l.debug(db))
                .field(&r.debug(db))
                .finish(),
            ExprData::Error => f.debug_tuple("Error").finish(),
        }
    }
}

id!(pub struct LocalVariableDecl);

impl DebugWithDb<InIrDb<'_, Tree>> for LocalVariableDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tree>) -> std::fmt::Result {
        DebugWithDb::fmt(self.data(db.tables()), f, db)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub struct LocalVariableDeclData {
    pub mode: Option<StorageMode>,
    pub name: Word,
    pub ty: Option<crate::ty::Ty>,
}

impl DebugWithDb<InIrDb<'_, Tree>> for LocalVariableDeclData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tree>) -> std::fmt::Result {
        f.debug_tuple("")
            .field(&self.name.debug(db.db()))
            .field(&self.ty.debug(db.db()))
            .finish()
    }
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct LocalVariableDeclSpan {
    pub mode_span: Span,
    pub name_span: Span,
}

id!(pub struct NamedExpr);

impl DebugWithDb<InIrDb<'_, Tree>> for NamedExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tree>) -> std::fmt::Result {
        DebugWithDb::fmt(self.data(db.tables()), f, db)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub struct NamedExprData {
    pub name: SpannedOptionalWord,
    pub expr: Expr,
}

impl DebugWithDb<InIrDb<'_, Tree>> for NamedExprData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &InIrDb<'_, Tree>) -> std::fmt::Result {
        f.debug_tuple("")
            .field(&self.name.word(db.db()).debug(db.db()))
            .field(&self.expr.debug(db))
            .finish()
    }
}

pub mod op;
