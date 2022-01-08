use crate::{
    parser::Parser,
    token_test::{FormatStringLiteral, Identifier, Number},
};

use dada_id::InternValue;
use dada_ir::{
    code::{
        syntax::{
            Expr, ExprData, LocalVariableDeclData, LocalVariableDeclSpan, NamedExpr, NamedExprData,
            Spans, Tables, Tree, TreeData,
        },
        Code,
    },
    format_string::FormatStringSectionData,
    kw::Keyword,
    op::Op,
    origin_table::PushOriginIn,
    span::Span,
    storage_mode::StorageMode,
    token::Token,
    token_tree::TokenTree,
    word::SpannedOptionalWord,
};
use salsa::AsId;

use super::{OrReportError, ParseList};

impl Parser<'_> {
    pub(crate) fn parse_syntax_tree(&mut self, origin: Code) -> Tree {
        let mut tables = Tables::default();
        let mut spans = Spans::default();

        let mut code_parser = CodeParser {
            parser: self,
            tables: &mut tables,
            spans: &mut spans,
        };

        let start = code_parser.tokens.last_span();
        let block = code_parser.parse_only_expr_seq();
        let span = code_parser.span_consumed_since(start);
        let root_expr = code_parser.add(ExprData::Seq(block), span);
        let tree_data = TreeData { tables, root_expr };
        Tree::new(self.db, origin, tree_data, spans)
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
        self.parser
    }
}

impl<'db> std::ops::DerefMut for CodeParser<'_, 'db> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.parser
    }
}

impl CodeParser<'_, '_> {
    /// Parses a series of expressions; expects to consume all available tokens (and errors if there are extra).
    #[tracing::instrument(level = "debug", skip(self))]
    pub(crate) fn parse_only_expr_seq(&mut self) -> Vec<Expr> {
        tracing::debug!("parse_only_expr_seq");
        let exprs = self.parse_list(true, CodeParser::parse_expr);
        tracing::debug!("exprs = {:?}", exprs);
        self.emit_error_if_more_tokens("extra tokens after end of expression");
        exprs
    }

    /// Parses a series of named expressions (`id: expr`); expects to consume all available tokens (and errors if there are extra).
    pub(crate) fn parse_only_named_exprs(&mut self) -> Vec<NamedExpr> {
        let exprs = self.parse_list(true, CodeParser::parse_named_expr);
        self.emit_error_if_more_tokens("extra tokens after end of arguments");
        exprs
    }

    fn add<D, K>(&mut self, data: D, span: K::Origin) -> K
    where
        D: std::hash::Hash + Eq + std::fmt::Debug,
        D: InternValue<Table = Tables, Key = K>,
        K: PushOriginIn<Spans> + AsId,
    {
        let key = self.tables.add(data);
        self.spans.push(key, span);
        key
    }

    /// Parses an if/while condition -- this can be any sort of expression but a block.
    pub(crate) fn parse_condition(&mut self) -> Option<Expr> {
        if self.peek(Token::Delimiter('{')).is_some() {
            None
        } else {
            self.parse_expr()
        }
    }

    ///
    pub(crate) fn parse_named_expr(&mut self) -> Option<NamedExpr> {
        let (label_span, label, expr);

        if let Some(spanned_label) = self.parse_label() {
            // If they provided `foo: ` then the expression is mandatory
            (label_span, label) = spanned_label;
            expr = self
                .parse_expr()
                .or_report_error(self, || "expected expression")
                .or_dummy_expr(self);
        } else {
            label_span = self.tokens.peek_span().span_at_start();
            expr = self.parse_expr()?;
            label = SpannedOptionalWord::new(self.db, None, label_span.in_file(self.filename));
        };

        Some(self.add(
            NamedExprData { name: label, expr },
            self.span_consumed_since(label_span),
        ))
    }

    /// Parse a `foo:` label.
    pub(crate) fn parse_label(&mut self) -> Option<(Span, SpannedOptionalWord)> {
        self.lookahead(|this| {
            let (name_span, name) = this.eat(Identifier)?;
            let _colon_span = this.eat_op(Op::Colon)?;
            Some((
                name_span,
                SpannedOptionalWord::new(this.db, Some(name), name_span.in_file(this.filename)),
            ))
        })
    }

    /// ```ignore
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
    ///       | SharingMode? Id = Expr
    /// ```
    #[tracing::instrument(level = "debug", skip(self))]
    pub(crate) fn parse_expr(&mut self) -> Option<Expr> {
        tracing::debug!("parse_expr");
        if let Some(expr) = self.parse_local_variable_decl() {
            return Some(expr);
        }

        self.parse_expr_3()
    }

    pub(crate) fn parse_expr_3(&mut self) -> Option<Expr> {
        let mut expr = self.parse_expr_2()?;

        loop {
            if let Some(expr1) = self.parse_binop(expr, &[Op::Plus, Op::Minus], Self::parse_expr_2)
            {
                expr = expr1;
                continue;
            }

            break;
        }

        Some(expr)
    }

    pub(crate) fn parse_expr_2(&mut self) -> Option<Expr> {
        let mut expr = self.parse_expr_1()?;

        loop {
            if let Some(expr1) =
                self.parse_binop(expr, &[Op::DividedBy, Op::Times], Self::parse_expr_1)
            {
                expr = expr1;
                continue;
            }

            break;
        }

        Some(expr)
    }

    pub(crate) fn parse_expr_1(&mut self) -> Option<Expr> {
        let mut expr = self.parse_expr_0()?;

        loop {
            if self.eat_op(Op::Dot).is_some() {
                if let Some((id_span, id)) = self.eat(Identifier) {
                    let span = self.spans[expr].to(id_span);
                    expr = self.add(ExprData::Dot(expr, id), span);
                    continue;
                } else if let Some((kw_span, _)) = self.eat(Keyword::Await) {
                    let span = self.spans[expr].to(kw_span);
                    expr = self.add(ExprData::Await(expr), span);
                    continue;
                } else if let Some((kw_span, _)) = self.eat(Keyword::Share) {
                    let span = self.spans[expr].to(kw_span);
                    expr = self.add(ExprData::Share(expr), span);
                    continue;
                } else if let Some((kw_span, _)) = self.eat(Keyword::Give) {
                    let span = self.spans[expr].to(kw_span);
                    expr = self.add(ExprData::Give(expr), span);
                    continue;
                } else if let Some((kw_span, _)) = self.eat(Keyword::Lease) {
                    let span = self.spans[expr].to(kw_span);
                    expr = self.add(ExprData::Lease(expr), span);
                    continue;
                } else {
                    self.parser
                        .error_at_current_token("expected identifier after `.`")
                        .emit(self.db);
                    continue;
                }
            }

            if let Some((arg_span, token_tree)) = self.delimited('(') {
                // `base(...)`
                let named_exprs = self
                    .with_sub_parser(token_tree, |sub_parser| sub_parser.parse_only_named_exprs());
                let span = self.spans[expr].to(arg_span);
                expr = self.add(ExprData::Call(expr, named_exprs), span);
                continue;
            }

            break;
        }

        Some(expr)
    }

    pub(crate) fn parse_expr_0(&mut self) -> Option<Expr> {
        if let Some((id_span, id)) = self.eat(Identifier) {
            Some(self.add(ExprData::Id(id), id_span))
        } else if let Some((word_span, word)) = self.eat(Number) {
            Some(self.add(ExprData::IntegerLiteral(word), word_span))
        } else if let Some(expr) = self.parse_format_string() {
            Some(expr)
        } else if let Some(expr) = self.parse_block_expr() {
            // { ... }
            Some(expr)
        } else if let Some((if_span, _)) = self.eat(Keyword::If) {
            if let Some(condition) = self.parse_condition() {
                let then_expr = self.parse_required_block_expr(Keyword::If);
                let else_expr = self
                    .eat(Keyword::Else)
                    .map(|_| self.parse_required_block_expr(Keyword::Else));
                let span = self.span_consumed_since(if_span);
                Some(self.add(ExprData::If(condition, then_expr, else_expr), span))
            } else {
                self.error_at_current_token("expected `if` condition")
                    .emit(self.db);
                None
            }
        } else if let Some((while_span, _)) = self.eat(Keyword::While) {
            if let Some(condition) = self.parse_condition() {
                let body = self.parse_required_block_expr(Keyword::While);
                let span = self.span_consumed_since(while_span);
                Some(self.add(ExprData::While(condition, body), span))
            } else {
                self.error_at_current_token("expected `while` condition")
                    .emit(self.db);
                None
            }
        } else if let Some((span, token_tree)) = self.delimited('(') {
            let expr =
                self.with_sub_parser(token_tree, |subparser| subparser.parse_only_expr_seq());
            Some(self.add(ExprData::Tuple(expr), span))
        } else {
            None
        }
    }

    /// Parses `[shared|var|atomic] x = expr`
    #[tracing::instrument(level = "debug", skip_all)]
    fn parse_local_variable_decl(&mut self) -> Option<Expr> {
        // A storage mode like `shared` or `var` *could* be a variable declaration,
        // but if we see `atomic` it might not be, so check for the `x = ` next.
        let (mode_span, mode) = if let Some(pair) = self.parse_storage_mode() {
            pair
        } else {
            (self.tokens.peek_span(), StorageMode::Shared)
        };

        // Look for `x = `. If we see that, we are committed to this
        // being a local variable declaration.
        let (name_span, name) = self.lookahead(|this| {
            let pair = this.eat(Identifier)?;
            this.eat_op(Op::Equal)?;
            Some(pair)
        })?;

        let local_variable_decl = self.add(
            LocalVariableDeclData {
                mode: Some(mode),
                name,
            },
            LocalVariableDeclSpan {
                mode_span,
                name_span,
            },
        );

        let value = self
            .parse_expr()
            .or_report_error(self, || "expected value for local variable".to_string())
            .or_dummy_expr(self);

        Some(self.add(
            ExprData::Var(local_variable_decl, value),
            self.span_consumed_since(mode_span),
        ))
    }

    fn parse_required_block_expr(&mut self, after: impl std::fmt::Display) -> Expr {
        self.parse_block_expr()
            .or_report_error(self, || format!("expected block after {after}"))
            .or_dummy_expr(self)
    }

    fn parse_block_expr(&mut self) -> Option<Expr> {
        let (span, token_tree) = self.delimited('{')?;
        let block = self.with_sub_parser(token_tree, |sub_parser| sub_parser.parse_only_expr_seq());
        let expr = self.add(ExprData::Seq(block), span);
        Some(expr)
    }

    fn parse_format_string(&mut self) -> Option<Expr> {
        let (span, format_string) = self.eat(FormatStringLiteral)?;

        // Special case for a string with no code like `"foo"`:
        if format_string.data(self.db).sections.len() == 1 {
            if let FormatStringSectionData::Text(word) =
                format_string.data(self.db).sections[0].data(self.db)
            {
                return Some(self.add(ExprData::StringLiteral(*word), span));
            }
        }

        todo!()
    }

    fn parse_binop(
        &mut self,
        base: Expr,
        ops: &[Op],
        mut parse_rhs: impl FnMut(&mut Self) -> Option<Expr>,
    ) -> Option<Expr> {
        for &op in ops {
            if self.eat_op(op).is_some() {
                let rhs = parse_rhs(self)
                    .or_report_error(self, || format!("expected expression after {op}"))
                    .or_dummy_expr(self);
                let span = self.spans[base].to(self.spans[rhs]);
                return Some(self.add(ExprData::Op(base, op, rhs), span));
            }
        }
        None
    }

    fn with_sub_parser<R>(
        &mut self,
        token_tree: TokenTree,
        op: impl FnOnce(&mut CodeParser<'_, '_>) -> R,
    ) -> R {
        let mut parser = Parser::new(self.db, token_tree);
        let mut sub_parser = CodeParser {
            parser: &mut parser,
            tables: self.tables,
            spans: self.spans,
        };
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || op(&mut sub_parser))
    }
}

trait OrDummyExpr {
    fn or_dummy_expr(self, parser: &mut CodeParser<'_, '_>) -> Expr;
}

impl OrDummyExpr for Option<Expr> {
    fn or_dummy_expr(self, parser: &mut CodeParser<'_, '_>) -> Expr {
        self.unwrap_or_else(|| parser.add(ExprData::Error, parser.tokens.peek_span()))
    }
}

impl ParseList for CodeParser<'_, '_> {
    fn skipped_newline(&self) -> bool {
        Parser::skipped_newline(self)
    }

    fn eat_comma(&mut self) -> bool {
        Parser::eat_comma(self)
    }
}
