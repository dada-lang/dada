use crate::{
    parser::Parser,
    token_test::{AnyKeyword, Identifier},
    tokens::Tokens,
};

use dada_id::InternValue;
use dada_ir::{
    code::{Ast, Block, BlockData, Expr, ExprData, NamedExpr, PushSpan, Spans, Tables},
    op::{BinaryOp, Op},
    token_tree::TokenTree,
};
use salsa::AsId;

impl Parser<'_> {
    pub(crate) fn parse_ast(&mut self) -> Ast {
        let mut tables = Tables::default();
        let mut spans = Spans::default();

        let mut code_parser = CodeParser {
            parser: self,
            tables: &mut tables,
            spans: &mut spans,
        };

        let block = code_parser.parse_block_contents();
        Ast { tables, block }
    }
}

struct CodeParser<'me, 'db> {
    parser: &'me mut Parser<'db>,
    tables: &'me mut Tables,
    spans: &'me mut Spans,
}

impl<'db> std::ops::Deref for CodeParser<'_, 'db> {
    type Target = Parser<'db>;

    fn deref(&self) -> &Self::Target {
        &self.parser
    }
}

impl<'db> std::ops::DerefMut for CodeParser<'_, 'db> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parser
    }
}

impl CodeParser<'_, '_> {
    pub(crate) fn parse_block_contents(&mut self) -> Block {
        let mut exprs = vec![];
        while self.tokens.peek().is_some() {
            let expr = self.parse_expr();
            exprs.push(expr);
        }
        self.tables.add(BlockData { exprs })
    }

    pub(crate) fn parse_named_exprs(&mut self) -> Vec<NamedExpr> {
        todo!()
    }

    fn add<D, K>(&mut self, data: D, span: K::Span) -> K
    where
        D: std::hash::Hash + Eq,
        Tables: InternValue<D, Key = K>,
        K: PushSpan + AsId,
    {
        let key = self.tables.add(data);
        key.push_span(&mut self.spans, span);
        key
    }

    /// ```
    /// Expr := Id
    ///       | UnaryOp Expr
    ///       | `if` Expr Block [`else` Block]
    ///       | `while` Expr Block
    ///       | `loop` Block
    ///       | `continue`
    ///       | `break` [Expr]
    ///       | `return` [Expr]
    ///       | Block
    ///       | Expr . Ident
    ///       | Expr BinaryOp Expr
    ///       | Expr ( args )
    /// ```
    pub(crate) fn parse_expr(&mut self) -> Expr {
        if let Some((id_span, id)) = self.eat_if(Identifier) {
            let expr = self.add(ExprData::Id(id), id_span);
            self.parse_expr_follow(expr)
        } else if let Some((span, token_tree)) = self.delimited('{') {
            // { ... }
            let block =
                self.with_sub_parser(token_tree, |sub_parser| sub_parser.parse_block_contents());
            let expr = self.add(ExprData::Block(block), span);
            self.parse_expr_follow(expr)
        } else if let Some((kw_span, kw)) = self.eat_if(AnyKeyword) {
            todo!()
        } else {
            todo!()
        }
    }

    fn parse_expr_follow(&mut self, mut base: Expr) -> Expr {
        'extend: loop {
            if let Some((_, _)) = self.eat_if(Op::Dot) {
                if let Some((id_span, id)) = self.eat_if(Identifier) {
                    let span = self.spans[base].to(id_span);
                    base = self.add(ExprData::Dot(base, id), span);
                    continue 'extend;
                } else {
                    self.parser
                        .report_error(self.tokens.peek_span(), "expected identifier after `.`");
                    continue 'extend;
                }
            }

            for &BinaryOp {
                binary_op,
                assign_op,
            } in dada_ir::op::binary_ops(self.db)
            {
                if let Some(_) = self.eat_if(binary_op) {
                    let rhs = self.parse_expr();
                    let span = self.spans[base].to(self.spans[rhs]);
                    base = self.add(ExprData::Op(base, binary_op, rhs), span);
                    continue 'extend;
                } else if let Some(_) = self.eat_if(assign_op) {
                    let rhs = self.parse_expr();
                    let span = self.spans[base].to(self.spans[rhs]);
                    base = self.add(ExprData::OpEq(base, binary_op, rhs), span);
                    continue 'extend;
                }
            }

            if let Some((arg_span, token_tree)) = self.delimited('(') {
                // `base(...)`
                let named_exprs =
                    self.with_sub_parser(token_tree, |sub_parser| sub_parser.parse_named_exprs());
                let span = self.spans[base].to(arg_span);
                base = self.add(ExprData::Call(base, named_exprs), span);
                continue 'extend;
            }

            return base;
        }
    }

    fn with_sub_parser<R>(
        &mut self,
        token_tree: TokenTree,
        op: impl FnOnce(&mut CodeParser<'_, '_>) -> R,
    ) -> R {
        let mut tokens = Tokens::new(self.db, token_tree);
        let mut parser = Parser::new(self.db, tokens, &mut self.parser.errors);
        let mut sub_parser = CodeParser {
            parser: &mut parser,
            tables: &mut self.tables,
            spans: &mut self.spans,
        };
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || op(&mut sub_parser))
    }
}
