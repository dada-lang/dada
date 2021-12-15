use crate::{op::Op, span::Span, storage_mode::StorageMode, token_tree::TokenTree, word::Word};
use dada_collections::IndexVec;
use dada_id::{id, tables};

salsa::entity2! {
    entity Code in crate::Jar {
        tokens: TokenTree,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ast {
    pub tables: Tables,
    pub block: Block,
}

#[derive(Default)]
pub struct Spans {
    pub expr_spans: IndexVec<Expr, Span>,
    pub named_expr_spans: IndexVec<NamedExpr, NamedExprSpan>,
    pub block_spans: IndexVec<Block, Span>,
}

impl<K> std::ops::Index<K> for Spans
where
    K: HasSpan,
{
    type Output = Span;

    fn index(&self, index: K) -> &Self::Output {
        index.span_in(self)
    }
}

pub trait HasSpan {
    fn span_in(self, spans: &Spans) -> &Span;
}

pub trait PushSpan {
    type Span;
    fn push_span(self, spans: &mut Spans, span: Self::Span);
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
    Dot(Expr, Word),
    Await(Expr),
    Call(Expr, Vec<NamedExpr>),
    Share(Expr),
    Give(Expr),
    Var(StorageMode, Word, Expr),
    If(Expr, Expr, Option<Expr>),

    // { ... } ==> closure?
    Block(Block),

    Op(Expr, Op, Expr),
    OpEq(Expr, Op, Expr),
    Assign(Expr, Expr),

    Error,
}

impl HasSpan for Expr {
    fn span_in(self, spans: &Spans) -> &Span {
        &spans.expr_spans[self]
    }
}

impl PushSpan for Expr {
    type Span = Span;

    fn push_span(self, spans: &mut Spans, span: Span) {
        spans.expr_spans.push(span);
        assert_eq!(Expr::from(spans.expr_spans.len()), self);
    }
}

id!(pub struct NamedExpr);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub struct NamedExprData {
    pub name: Word,
    pub expr: Expr,
}

pub struct NamedExprSpan {
    pub span: Span,
    pub name_span: Span,
}

impl HasSpan for NamedExpr {
    fn span_in(self, spans: &Spans) -> &Span {
        &spans.named_expr_spans[self].span
    }
}

impl PushSpan for NamedExpr {
    type Span = NamedExprSpan;

    fn push_span(self, spans: &mut Spans, span: NamedExprSpan) {
        spans.named_expr_spans.push(span);
        assert_eq!(NamedExpr::from(spans.named_expr_spans.len()), self);
    }
}

id!(pub struct Block);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub struct BlockData {
    pub exprs: Vec<Expr>,
}

impl HasSpan for Block {
    fn span_in(self, spans: &Spans) -> &Span {
        &spans.block_spans[self]
    }
}

impl PushSpan for Block {
    type Span = Span;

    fn push_span(self, spans: &mut Spans, span: Span) {
        spans.block_spans.push(span);
        assert_eq!(Block::from(spans.block_spans.len()), self);
    }
}
