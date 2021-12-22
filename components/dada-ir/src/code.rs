use crate::{
    op::Op, span::Span, span_table::EntireSpan, storage_mode::StorageMode, token_tree::TokenTree,
    word::Word,
};
use dada_collections::IndexVec;
use dada_id::{id, tables};

salsa::entity2! {
    /// "Code" represents a block of code attached to a method.
    /// After parsing, it just contains a token tree, but you can...
    ///
    /// * use the `ast` method from the `dada_parse` prelude to
    ///   parse it into an `Ast`.
    entity Code in crate::Jar {
        tokens: TokenTree,
    }
}

/// Stores the ast for a function.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ast {
    /// Interning tables for expressions and the like.
    pub tables: Tables,

    /// The root
    pub block: Block,
}

span_table! {
    /// Side table that contains the spans for everything in an AST.
    /// This isn't normally needed except for diagnostics, so it's
    /// kept separate to avoid reducing incremental reuse.
    /// You can request it by invoking the `spans`
    /// method in the `dada_parse` prelude.
    #[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
    pub struct Spans {
        expr_spans: Expr => Span,
        named_expr_spans: NamedExpr => NamedExprSpan,
        block_spans: Block => Span,
    }
}

tables! {
    pub struct Tables {
        exprs: alloc Expr => ExprData,
        named_exprs: alloc NamedExpr => NamedExprData,
        blocks: alloc Block => BlockData,
    }
}

id!(pub struct Expr);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub enum ExprData {
    Id(Word),

    /// `"foo"` with no format strings
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
    Var(StorageMode, Word, Expr),

    /// `(expr)`
    Parenthesized(Expr),

    /// `if condition { block } [else { block }]`
    If(Expr, Expr, Option<Expr>),

    /// `loop { block }`
    Loop(Expr),

    /// `while condition { block }`
    While(Expr, Expr),

    // { ... } ==> closure?
    Block(Block),

    /// `a + b`
    Op(Expr, Op, Expr),

    /// `a += b`
    OpEq(Expr, Op, Expr),

    /// `a := b`
    Assign(Expr, Expr),

    /// parse or other error
    Error,
}

id!(pub struct NamedExpr);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub struct NamedExprData {
    pub name: Word,
    pub expr: Expr,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NamedExprSpan {
    pub span: Span,
    pub name_span: Span,
}

impl EntireSpan for NamedExprSpan {
    fn entire_span(&self) -> &Span {
        &self.span
    }
}

id!(pub struct Block);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub struct BlockData {
    pub exprs: Vec<Expr>,
}
