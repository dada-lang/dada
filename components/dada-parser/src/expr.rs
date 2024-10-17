use dada_ir_ast::ast::{
    AstConstructorField, AstExpr, AstExprKind, AstPath, BinaryOp, DeferredParse, Literal,
    SpannedBinaryOp, SquareBracketArgs,
};

use crate::{
    tokenizer::{Keyword, Token, TokenKind},
    Parse, Parser,
};

impl<'db> Parse<'db> for AstExpr<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self::Output>, crate::ParseFail<'db>> {
        opt_parse_expr_with_precedence(db, parser, binary_expr_precedence)
    }

    fn expected() -> crate::Expected {
        crate::Expected::Nonterminal("expression")
    }
}

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
    Ok(Some(AstExpr::new(start_span.to(parser.last_span()), kind)))
}

const BINARY_OP_PRECEDENCE: &[&[(&str, BinaryOp)]] = &[
    &[("+", BinaryOp::Add), ("-", BinaryOp::Sub)],
    &[("*", BinaryOp::Mul), ("*", BinaryOp::Div)],
];

fn binary_expr_precedence<'db>(
    db: &'db dyn crate::Db,
    parser: &mut Parser<'_, 'db>,
) -> Result<Option<AstExprKind<'db>>, crate::ParseFail<'db>> {
    binary_expr_with_precedence_level(db, parser, 0)
}

fn binary_expr_with_precedence_level<'db>(
    db: &'db dyn crate::Db,
    parser: &mut Parser<'_, 'db>,
    precedence: usize,
) -> Result<Option<AstExprKind<'db>>, crate::ParseFail<'db>> {
    let start_span = parser.peek_span();

    if precedence >= BINARY_OP_PRECEDENCE.len() {
        return Ok(postfix_expr_precedence(db, parser)?);
    }

    // Parse the LHS at one higher level of precedence than
    // the current one.
    let Some(mut lhs_kind) = binary_expr_with_precedence_level(db, parser, precedence + 1)? else {
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
                    let lhs = AstExpr::new(start_span.to(mid_span), lhs_kind);
                    let rhs = eat_expr_with_precedence(db, parser, |db, parser| {
                        // Parse RHS at the current level of precedence:
                        binary_expr_with_precedence_level(db, parser, precedence)
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

fn postfix_expr_precedence<'db>(
    db: &'db dyn crate::Db,
    parser: &mut Parser<'_, 'db>,
) -> Result<Option<AstExprKind<'db>>, crate::ParseFail<'db>> {
    let start_span = parser.peek_span();

    let Some(mut kind) = base_expr_precedence(db, parser)? else {
        return Ok(None);
    };

    loop {
        let mid_span = parser.peek_span();

        // `.` can skip newlines
        if let Ok(_) = parser.eat_op(".") {
            let id = parser.eat_id()?;
            let owner = AstExpr::new(start_span.to(mid_span), kind);
            kind = AstExprKind::DotId(owner, id);
            continue;
        }

        // Postfix `[]` is only valid on the same line, since `[..]` is also valid as the start of an expression
        if parser.next_token_on_same_line() {
            if let Ok(text) = parser.eat_delimited(crate::tokenizer::Delimiter::SquareBrackets) {
                let owner = AstExpr::new(start_span.to(mid_span), kind);
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
                let owner = AstExpr::new(start_span.to(mid_span), kind);
                kind = AstExprKind::ParenthesisOp(owner, args);
                continue;
            }
        }

        return Ok(Some(kind));
    }
}

fn base_expr_precedence<'db>(
    db: &'db dyn crate::Db,
    parser: &mut Parser<'_, 'db>,
) -> Result<Option<AstExprKind<'db>>, crate::ParseFail<'db>> {
    if let Some(literal) = Literal::opt_parse(db, parser)? {
        return Ok(Some(AstExprKind::Literal(literal)));
    }

    if let Ok(id) = parser.eat_id() {
        // Could be `X { field1: value1, .. }`
        if parser.next_token_on_same_line() {
            if let Some(fields) = AstConstructorField::opt_parse_delimited(
                db,
                parser,
                crate::tokenizer::Delimiter::CurlyBraces,
                AstConstructorField::eat_comma,
            )? {
                let path = AstPath::new(db, vec![id]);
                return Ok(Some(AstExprKind::Constructor(path, fields)));
            }
        }

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

    Ok(None)
}

impl<'db> Parse<'db> for Literal<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self::Output>, crate::ParseFail<'db>> {
        let Some(Token {
            kind: TokenKind::Literal(kind, text),
            ..
        }) = parser.peek()
        else {
            return Ok(None);
        };

        let literal = Literal::new(db, *kind, text.to_string());

        parser.eat_next_token().unwrap();

        Ok(Some(literal))
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

        let _colon = parser.eat_op(":")?;

        let value = AstExpr::eat(db, parser)?;

        Ok(Some(AstConstructorField { name, value }))
    }

    fn expected() -> crate::Expected {
        crate::Expected::Nonterminal("field initializer")
    }
}
