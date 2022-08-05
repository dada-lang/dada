use crate::{
    parser::Parser,
    token_test::{Alphabetic, FormatStringLiteral, Identifier, Number},
};

use dada_id::InternValue;
use dada_ir::{
    code::{
        syntax::{op::Op, LocalVariableDecl},
        syntax::{
            Expr, ExprData, LocalVariableDeclData, LocalVariableDeclSpan, NamedExpr, NamedExprData,
            Spans, Tables, Tree, TreeData,
        },
    },
    format_string::FormatStringSectionData,
    kw::Keyword,
    origin_table::PushOriginIn,
    parameter::Parameter,
    span::Span,
    storage::Atomic,
    token::Token,
    token_tree::TokenTree,
    word::SpannedOptionalWord,
};
use salsa::AsId;

use super::{OrReportError, ParseList};

impl Parser<'_> {
    pub(crate) fn parse_code_body(&mut self, parameters: &[Parameter]) -> Tree {
        let db = self.db;
        let mut tables = Tables::default();
        let mut spans = Spans::default();

        let mut code_parser = CodeParser {
            parser: self,
            tables: &mut tables,
            spans: &mut spans,
        };

        let parameter_decls = parameters
            .iter()
            .map(|parameter| code_parser.add(parameter.decl(db), parameter.decl_span(db)))
            .collect::<Vec<_>>();

        let start = code_parser.tokens.last_span();
        let exprs = code_parser.parse_only_expr_seq();
        self.create_syntax_tree(start, parameter_decls, tables, spans, exprs)
    }

    pub(crate) fn parse_top_level_expr(
        &mut self,
        tables: &mut Tables,
        spans: &mut Spans,
    ) -> Option<Expr> {
        let mut code_parser = CodeParser {
            parser: self,
            tables,
            spans,
        };
        code_parser.parse_expr()
    }

    pub(crate) fn create_syntax_tree(
        &mut self,
        start: Span,
        parameter_decls: Vec<LocalVariableDecl>,
        mut tables: Tables,
        mut spans: Spans,
        exprs: Vec<Expr>,
    ) -> Tree {
        let span = self.span_consumed_since(start);

        let root_expr = {
            let mut code_parser = CodeParser {
                parser: self,
                tables: &mut tables,
                spans: &mut spans,
            };
            code_parser.add(ExprData::Seq(exprs), span)
        };

        let tree_data = TreeData {
            tables,
            parameter_decls,
            root_expr,
        };
        Tree::new(self.db, tree_data, spans)
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

    /// Parses a series of expressions; expects to consume all available tokens (and errors if there are extra).
    fn parse_only_expr(&mut self) -> Option<Expr> {
        let expr = self.parse_expr()?;
        self.emit_error_if_more_tokens("extra tokens after end of expression");
        Some(expr)
    }

    /// Parses a series of named expressions (`id: expr`); expects to consume all available tokens (and errors if there are extra).
    pub(crate) fn parse_only_named_exprs(&mut self) -> Vec<NamedExpr> {
        let exprs = self.parse_list(true, CodeParser::parse_named_expr);
        self.emit_error_if_more_tokens("extra tokens after end of arguments");
        exprs
    }

    fn add<D, K>(&mut self, data: D, mut span: K::Origin) -> K
    where
        D: std::hash::Hash + Eq + std::fmt::Debug,
        D: InternValue<Table = Tables, Key = K>,
        K: PushOriginIn<Spans> + AsId,
        K::Origin: TightenSpan,
    {
        let key = self.tables.add(data);
        span = span.tighten_span(self);
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
            label = SpannedOptionalWord::new(self.db, None, label_span.in_file(self.input_file));
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
                SpannedOptionalWord::new(this.db, Some(name), name_span.in_file(this.input_file)),
            ))
        })
    }

    /// ```text
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

        if let Some((return_span, _)) = self.eat(Keyword::Return) {
            match self.parse_expr() {
                Some(expr) => {
                    let span = self.span_consumed_since(return_span);
                    return Some(self.add(ExprData::Return(Some(expr)), span));
                }
                None => {
                    return Some(self.add(ExprData::Return(None), return_span));
                }
            }
        }

        self.parse_expr_6()
    }

    pub(crate) fn parse_expr_6(&mut self) -> Option<Expr> {
        let mut expr = self.parse_expr_5()?;

        loop {
            if let Some(expr1) = self.parse_binop(
                expr,
                &[
                    Op::PlusEqual,
                    Op::MinusEqual,
                    Op::DividedByEqual,
                    Op::TimesEqual,
                    Op::ColonEqual,
                ],
                Self::parse_expr_5,
            ) {
                expr = expr1;
                continue;
            }

            break;
        }

        Some(expr)
    }

    pub(crate) fn parse_expr_5(&mut self) -> Option<Expr> {
        let mut expr = self.parse_expr_4()?;

        loop {
            if let Some(expr1) = self.parse_binop(
                expr,
                &[
                    Op::EqualEqual,
                    Op::LessThan,
                    Op::GreaterThan,
                    Op::GreaterEqual,
                    Op::LessEqual,
                ],
                Self::parse_expr_4,
            ) {
                expr = expr1;
                continue;
            }

            break;
        }

        Some(expr)
    }

    pub(crate) fn parse_expr_4(&mut self) -> Option<Expr> {
        let mut expr = self.parse_expr_3()?;

        loop {
            if let Some(expr1) = self.parse_binop(expr, &[Op::Plus, Op::Minus], Self::parse_expr_3)
            {
                expr = expr1;
                continue;
            }

            break;
        }

        Some(expr)
    }

    pub(crate) fn parse_expr_3(&mut self) -> Option<Expr> {
        let mut expr = self.parse_expr_2()?;

        loop {
            if let Some(expr1) =
                self.parse_binop(expr, &[Op::DividedBy, Op::Times], Self::parse_expr_2)
            {
                expr = expr1;
                continue;
            }

            break;
        }

        Some(expr)
    }

    pub(crate) fn parse_expr_2(&mut self) -> Option<Expr> {
        if let Some(expr) = self.parse_unary(&[Op::Minus], Self::parse_expr_2) {
            return Some(expr);
        }
        self.parse_expr_1()
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
        tracing::debug!("parse_expr_0: peek = {:?}", self.tokens.peek());
        if let Some((true_span, _)) = self.eat(Keyword::True) {
            Some(self.add(ExprData::BooleanLiteral(true), true_span))
        } else if let Some((false_span, _)) = self.eat(Keyword::False) {
            Some(self.add(ExprData::BooleanLiteral(false), false_span))
        } else if let Some((id_span, id)) = self.eat(Identifier) {
            tracing::debug!("identifier");
            Some(self.add(ExprData::Id(id), id_span))
        } else if let Some((word_span, word)) = self.eat(Number) {
            let whitespace_after_number = self.tokens.skipped_any();

            match self.eat_op(Op::Dot) {
                None => {
                    if whitespace_after_number {
                        return Some(self.add(ExprData::IntegerLiteral(word, None), word_span));
                    }
                    match self.eat(Alphabetic) {
                        Some((_, alphabetic)) => {
                            let span = self.span_consumed_since(word_span);
                            Some(self.add(ExprData::IntegerLiteral(word, Some(alphabetic)), span))
                        }
                        None => Some(self.add(ExprData::IntegerLiteral(word, None), word_span)),
                    }
                }
                Some(dot_span) => {
                    let whitespace_after_dot = self.tokens.skipped_any();
                    if let Some((_, dec_word)) = self.eat(Number) {
                        let span = self.span_consumed_since(word_span);

                        if whitespace_after_number || whitespace_after_dot {
                            self.parser
                                .error(span, "whitespace is not allowed in float literals")
                                .emit(self.db);
                        }

                        Some(self.add(ExprData::FloatLiteral(word, dec_word), span))
                    } else {
                        self.parser
                            .error(dot_span, "expected digits after `.`")
                            .emit(self.db);
                        let span = self.span_consumed_since(word_span);
                        Some(self.add(ExprData::Error, span))
                    }
                }
            }
        } else if let Some(expr) = self.parse_format_string() {
            Some(expr)
        } else if let Some(expr) = self.parse_block_expr() {
            // { ... }
            Some(expr)
        } else if let Some((kw_span, _)) = self.eat(Keyword::Atomic) {
            let body_expr = self.parse_required_block_expr(Keyword::Atomic);
            let span = self.span_consumed_since(kw_span);
            tracing::debug!("atomic");
            Some(self.add(ExprData::Atomic(body_expr), span))
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
        } else if let Some((loop_span, _)) = self.eat(Keyword::Loop) {
            let body = self.parse_required_block_expr(Keyword::Loop);
            let span = self.span_consumed_since(loop_span);
            Some(self.add(ExprData::Loop(body), span))
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

            Some(self.add(
                if expr.len() == 1 {
                    ExprData::Parenthesized(expr[0])
                } else {
                    ExprData::Tuple(expr)
                },
                span,
            ))
        } else {
            None
        }
    }

    /// Parses `[permission-mode] [atomic] x = expr`
    #[tracing::instrument(level = "debug", skip_all)]
    fn parse_local_variable_decl(&mut self) -> Option<Expr> {
        // Look for `[mode] x = `. If we see that, we are committed to this
        // being a local variable declaration. Otherwise, we roll fully back.
        let (atomic_span, atomic, name_span, name) = self.lookahead(|this| {
            // A storage mode like `shared` or `var` *could* be a variable declaration,
            // but if we see `atomic` it might not be, so check for the `x = ` next.
            let (atomic_span, atomic) = if let Some(span) = this.parse_atomic() {
                (span, Atomic::Yes)
            } else {
                (this.tokens.peek_span(), Atomic::No)
            };

            let (name_span, name) = this.eat(Identifier)?;

            this.eat_op(Op::Equal)?;

            Some((atomic_span, atomic, name_span, name))
        })?;

        let local_variable_decl = self.add(
            LocalVariableDeclData {
                atomic,
                name,
                ty: None, // FIXME-- should permit `ty: Ty = ...`
            },
            LocalVariableDeclSpan {
                atomic_span,
                name_span,
            },
        );

        let value = self
            .parse_expr()
            .or_report_error(self, || "expected value for local variable".to_string())
            .or_dummy_expr(self);

        Some(self.add(
            ExprData::Var(local_variable_decl, value),
            self.span_consumed_since(atomic_span),
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

    fn parse_required_sub_expr(&mut self, token_tree: TokenTree) -> Expr {
        let db = self.db;
        self.with_sub_parser(token_tree, |sub_parser| sub_parser.parse_only_expr())
            .or_report_error_at(self, token_tree.span(db), || {
                "expected expression here".to_string()
            })
            .or_dummy_expr(self)
    }

    fn parse_format_string(&mut self) -> Option<Expr> {
        let (span, format_string) = self.eat(FormatStringLiteral)?;

        let exprs: Vec<Expr> = format_string
            .sections(self.db)
            .iter()
            .map(|section| match section.data(self.db) {
                FormatStringSectionData::Text(word) => {
                    self.add(ExprData::StringLiteral(word), span)
                }
                FormatStringSectionData::TokenTree(tree) => self.parse_required_sub_expr(tree),
            })
            .collect();

        Some(self.add(ExprData::Concatenate(exprs), span))
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
                match op {
                    Op::ColonEqual => return Some(self.add(ExprData::Assign(base, rhs), span)),
                    Op::PlusEqual | Op::MinusEqual | Op::DividedByEqual | Op::TimesEqual => {
                        return Some(self.add(ExprData::OpEq(base, op, rhs), span))
                    }
                    _ => return Some(self.add(ExprData::Op(base, op, rhs), span)),
                }
            }
        }
        None
    }

    fn parse_unary(
        &mut self,
        ops: &[Op],
        mut parse_rhs: impl FnMut(&mut Self) -> Option<Expr>,
    ) -> Option<Expr> {
        for &op in ops {
            if let Some(op_span) = self.eat_op(op) {
                let rhs = parse_rhs(self)
                    .or_report_error(self, || format!("expected expression after {op}"))
                    .or_dummy_expr(self);
                let span = self.span_consumed_since(op_span);
                return Some(self.add(ExprData::Unary(op, rhs), span));
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
        op(&mut sub_parser)
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

trait TightenSpan {
    fn tighten_span(self, parser: &Parser<'_>) -> Self;
}

impl TightenSpan for Span {
    fn tighten_span(self, parser: &Parser<'_>) -> Self {
        parser.tighten_span(self)
    }
}

impl TightenSpan for LocalVariableDeclSpan {
    fn tighten_span(self, parser: &Parser<'_>) -> Self {
        LocalVariableDeclSpan {
            atomic_span: self.atomic_span.tighten_span(parser),
            name_span: self.name_span.tighten_span(parser),
        }
    }
}
