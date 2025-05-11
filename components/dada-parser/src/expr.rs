use dada_ir_ast::ast::{
    AstBinaryOp, AstBlock, AstConstructorField, AstExpr, AstExprKind, AstPath, AstPathKind,
    DeferredParse, Identifier, IfArm, Literal, LiteralKind, PermissionOp, SpannedBinaryOp,
    SpannedIdentifier, SpannedUnaryOp, SquareBracketArgs, UnaryOp,
};

use crate::{
    Parse, Parser,
    tokenizer::{
        Keyword, Token, TokenKind,
        operator::{self, Op},
    },
};

impl<'db> Parse<'db> for AstExpr<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self::Output>, crate::ParseFail<'db>> {
        opt_parse_expr_with_precedence(db, parser, binary_expr_precedence::<SELECT_ALL>)
    }

    fn expected() -> crate::Expected {
        crate::Expected::Nonterminal("expression")
    }
}

const SELECT_ALL: u32 = u32::MAX;
const SELECT_STRUCT: u32 = 1;

fn eat_expr_with_precedence<'db>(
    db: &'db dyn crate::Db,
    parser: &mut Parser<'_, 'db>,
    precedence: impl FnOnce(
        &'db dyn crate::Db,
        &mut Parser<'_, 'db>,
    ) -> Result<Option<AstExprKind<'db>>, crate::ParseFail<'db>>,
) -> Result<AstExpr<'db>, crate::ParseFail<'db>> {
    match opt_parse_expr_with_precedence(db, parser, precedence)? {
        Some(e) => Ok(e),
        None => Err(parser.illformed(AstExpr::expected())),
    }
}

fn opt_parse_expr_with_precedence<'db>(
    db: &'db dyn crate::Db,
    parser: &mut Parser<'_, 'db>,
    precedence: impl FnOnce(
        &'db dyn crate::Db,
        &mut Parser<'_, 'db>,
    ) -> Result<Option<AstExprKind<'db>>, crate::ParseFail<'db>>,
) -> Result<Option<AstExpr<'db>>, crate::ParseFail<'db>> {
    let start_span = parser.peek_span();
    let Some(kind) = precedence(db, parser)? else {
        return Ok(None);
    };
    Ok(Some(AstExpr::new(
        start_span.to(db, parser.last_span()),
        kind,
    )))
}

const BINARY_OP_PRECEDENCE: &[&[(Op, AstBinaryOp)]] = &[
    &[
        (operator::PLUS, AstBinaryOp::Add),
        (operator::MINUS, AstBinaryOp::Sub),
    ],
    &[
        (operator::STAR, AstBinaryOp::Mul),
        (operator::SLASH, AstBinaryOp::Div),
    ],
    &[
        (operator::GREATERTHANEQ, AstBinaryOp::GreaterEqual),
        (operator::LESSTHANEQ, AstBinaryOp::LessEqual),
        (operator::GREATERTHAN, AstBinaryOp::GreaterThan),
        (operator::LESSTHAN, AstBinaryOp::LessThan),
        (operator::EQEQ, AstBinaryOp::EqualEqual),
    ],
    &[(operator::ANDAND, AstBinaryOp::AndAnd)],
    &[(operator::PIPEPIPE, AstBinaryOp::OrOr)],
    &[(operator::EQ, AstBinaryOp::Assign)],
];

fn binary_expr_precedence<'db, const SELECT: u32>(
    db: &'db dyn crate::Db,
    parser: &mut Parser<'_, 'db>,
) -> Result<Option<AstExprKind<'db>>, crate::ParseFail<'db>> {
    binary_expr_with_precedence_level::<SELECT>(db, parser, 0)
}

fn binary_expr_with_precedence_level<'db, const SELECT: u32>(
    db: &'db dyn crate::Db,
    parser: &mut Parser<'_, 'db>,
    precedence: usize,
) -> Result<Option<AstExprKind<'db>>, crate::ParseFail<'db>> {
    let start_span = parser.peek_span();

    if precedence >= BINARY_OP_PRECEDENCE.len() {
        return postfix_expr_precedence::<SELECT>(db, parser);
    }

    // Parse the LHS at one higher level of precedence than
    // the current one.
    let Some(mut lhs_kind) =
        binary_expr_with_precedence_level::<SELECT>(db, parser, precedence + 1)?
    else {
        return Ok(None);
    };

    // Parse as many RHS at the current level of precedence as we can find.
    // Note that the binary operator must appear on the current line;
    // binary operators on the *next line* don't count, those are prefix unary operators (or errors,
    // as the case may be).
    'outer: loop {
        let mid_span = parser.peek_span();

        if parser.next_token_on_same_line() {
            for &(op_text, op) in BINARY_OP_PRECEDENCE[precedence] {
                if let Ok(op_span) = parser.eat_op(op_text) {
                    let lhs = AstExpr::new(start_span.to(db, mid_span), lhs_kind);
                    let rhs = eat_expr_with_precedence(db, parser, |db, parser| {
                        // Parse RHS at the current level of precedence:
                        binary_expr_with_precedence_level::<SELECT>(db, parser, precedence)
                    })?;
                    lhs_kind =
                        AstExprKind::BinaryOp(SpannedBinaryOp { span: op_span, op }, lhs, rhs);
                    continue 'outer;
                }
            }
        }

        return Ok(Some(lhs_kind));
    }
}

fn postfix_expr_precedence<'db, const SELECT: u32>(
    db: &'db dyn crate::Db,
    parser: &mut Parser<'_, 'db>,
) -> Result<Option<AstExprKind<'db>>, crate::ParseFail<'db>> {
    let start_span = parser.peek_span();

    let Some(mut kind) = base_expr_precedence::<SELECT>(db, parser)? else {
        return Ok(None);
    };

    loop {
        let mid_span = parser.last_span();

        // `.` can skip newlines
        if parser.eat_op(operator::DOT).is_ok() {
            if let Ok(id) = parser.eat_id() {
                let owner = AstExpr::new(start_span.to(db, mid_span), kind);
                kind = AstExprKind::DotId(owner, id);
                continue;
            }

            if let Ok(await_keyword) = parser.eat_keyword(Keyword::Await) {
                let future = AstExpr::new(start_span.to(db, mid_span), kind);
                kind = AstExprKind::Await {
                    future,
                    await_keyword,
                };
                continue;
            }

            if let Some(op) = PermissionOp::opt_parse(db, parser)? {
                let value = AstExpr::new(start_span.to(db, mid_span), kind);
                kind = AstExprKind::PermissionOp { value, op };
                continue;
            }
        }

        // Postfix `[]` is only valid on the same line, since `[..]` is also valid as the start of an expression
        if parser.next_token_on_same_line() {
            if let Ok(text) = parser.eat_delimited(crate::tokenizer::Delimiter::SquareBrackets) {
                let owner = AstExpr::new(start_span.to(db, mid_span), kind);
                let deferred = DeferredParse {
                    span: parser.last_span(),
                    contents: text.to_string(),
                };
                let args = SquareBracketArgs::new(db, deferred);
                kind = AstExprKind::SquareBracketOp(owner, args);
                continue;
            }
        }

        // Postfix `()` is only valid on the same line, since `[..]` is also valid as the start of an expression
        if parser.next_token_on_same_line() {
            if let Some(args) = AstExpr::opt_parse_delimited(
                db,
                parser,
                crate::tokenizer::Delimiter::Parentheses,
                AstExpr::eat_comma,
            )? {
                let owner = AstExpr::new(start_span.to(db, mid_span), kind);
                kind = AstExprKind::ParenthesisOp(owner, args);
                continue;
            }
        }

        return Ok(Some(kind));
    }
}

fn base_expr_precedence<'db, const SELECT: u32>(
    db: &'db dyn crate::Db,
    parser: &mut Parser<'_, 'db>,
) -> Result<Option<AstExprKind<'db>>, crate::ParseFail<'db>> {
    if let Some(literal) = Literal::opt_parse(db, parser)? {
        return Ok(Some(AstExprKind::Literal(literal)));
    }

    if let Ok(if_span) = parser.eat_keyword(Keyword::If) {
        return Ok(Some(if_chain(db, parser, if_span)?));
    }

    if let Ok(id) = parser.eat_id() {
        // Could be `X { field1: value1, .. }`
        if (SELECT & SELECT_STRUCT != 0) && parser.next_token_on_same_line() {
            if let Some(fields) = AstConstructorField::opt_parse_delimited(
                db,
                parser,
                crate::tokenizer::Delimiter::CurlyBraces,
                AstConstructorField::eat_comma,
            )? {
                let path = AstPath::new(db, AstPathKind::Identifier(id));
                return Ok(Some(AstExprKind::Constructor(path, fields)));
            }
        }

        return Ok(Some(AstExprKind::Id(id)));
    }

    if let Ok(span) = parser.eat_keyword(Keyword::Self_) {
        let id = SpannedIdentifier {
            span,
            id: Identifier::self_ident(db),
        };
        return Ok(Some(AstExprKind::Id(id)));
    }

    if parser.eat_keyword(Keyword::Return).is_ok() {
        // Could be `return foo`
        if parser.next_token_on_same_line() {
            if let Some(expr) = AstExpr::opt_parse(db, parser)? {
                return Ok(Some(AstExprKind::Return(Some(expr))));
            }
        }
        return Ok(Some(AstExprKind::Return(None)));
    }

    if let Ok(span) = parser.eat_op(operator::BANG) {
        let expr = eat_expr_with_precedence(db, parser, postfix_expr_precedence::<SELECT>)?;
        return Ok(Some(AstExprKind::UnaryOp(
            SpannedUnaryOp {
                span,
                op: UnaryOp::Not,
            },
            expr,
        )));
    }

    if let Ok(span) = parser.eat_op(operator::MINUS) {
        let expr = eat_expr_with_precedence(db, parser, postfix_expr_precedence::<SELECT>)?;
        return Ok(Some(AstExprKind::UnaryOp(
            SpannedUnaryOp {
                span,
                op: UnaryOp::Negate,
            },
            expr,
        )));
    }

    Ok(None)
}

fn if_chain<'db>(
    db: &'db dyn crate::Db,
    parser: &mut Parser<'_, 'db>,
    _if_span: dada_ir_ast::span::Span<'db>,
) -> Result<AstExprKind<'db>, crate::ParseFail<'db>> {
    let condition0 = eat_expr_with_precedence(
        db,
        parser,
        binary_expr_precedence::<{ SELECT_ALL - SELECT_STRUCT }>,
    )?;

    let block0 = AstBlock::eat(db, parser)?;

    let mut arms = vec![IfArm {
        condition: Some(condition0),
        result: block0,
    }];

    loop {
        let Ok(_else_span) = parser.eat_keyword(Keyword::Else) else {
            break;
        };

        if let Ok(_if_span) = parser.eat_keyword(Keyword::If) {
            let else_if_condition = eat_expr_with_precedence(
                db,
                parser,
                binary_expr_precedence::<{ SELECT_ALL - SELECT_STRUCT }>,
            )?;
            let else_if_block = AstBlock::eat(db, parser)?;
            arms.push(IfArm {
                condition: Some(else_if_condition),
                result: else_if_block,
            });
        } else {
            let else_block = AstBlock::eat(db, parser)?;
            arms.push(IfArm {
                condition: None,
                result: else_block,
            });
            break;
        }
    }

    Ok(AstExprKind::If(arms))
}

impl<'db> Parse<'db> for PermissionOp {
    type Output = Self;

    fn opt_parse(
        _db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self::Output>, crate::ParseFail<'db>> {
        if parser.eat_keyword(Keyword::Give).is_ok() {
            Ok(Some(PermissionOp::Give))
        } else if parser.eat_keyword(Keyword::Mut).is_ok() {
            Ok(Some(PermissionOp::Mutate))
        } else if parser.eat_keyword(Keyword::Ref).is_ok() {
            Ok(Some(PermissionOp::Reference))
        } else {
            Ok(None)
        }
    }

    fn expected() -> crate::Expected {
        crate::Expected::Nonterminal("permission operator")
    }
}

impl<'db> Parse<'db> for Literal<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self::Output>, crate::ParseFail<'db>> {
        match parser.peek() {
            Some(Token {
                kind: TokenKind::Literal(kind, text),
                ..
            }) => {
                let literal = Literal::new(db, *kind, text.to_string());

                parser.eat_next_token().unwrap();

                Ok(Some(literal))
            }

            Some(Token {
                kind: TokenKind::Keyword(Keyword::True),
                ..
            }) => {
                parser.eat_next_token().unwrap();
                Ok(Some(Literal::new(db, LiteralKind::Boolean, "true")))
            }

            Some(Token {
                kind: TokenKind::Keyword(Keyword::False),
                ..
            }) => {
                parser.eat_next_token().unwrap();
                Ok(Some(Literal::new(db, LiteralKind::Boolean, "false")))
            }

            _ => Ok(None),
        }
    }

    fn expected() -> crate::Expected {
        crate::Expected::Nonterminal("literal")
    }
}

impl<'db> Parse<'db> for AstConstructorField<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self::Output>, crate::ParseFail<'db>> {
        let Ok(name) = parser.eat_id() else {
            return Ok(None);
        };

        let _colon = parser.eat_op(operator::COLON)?;

        let value = AstExpr::eat(db, parser)?;

        Ok(Some(AstConstructorField { name, value }))
    }

    fn expected() -> crate::Expected {
        crate::Expected::Nonterminal("field initializer")
    }
}
