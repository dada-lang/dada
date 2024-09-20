use dada_ir_ast::ast::{
    AstConstructorField, AstExpr, AstExprKind, Literal, Path, SquareBracketArgs,
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
        let start_span = parser.peek_span();
        let Some(kind) = postfix_expr_kind(db, parser)? else {
            return Ok(None);
        };
        Ok(Some(AstExpr::new(start_span.to(parser.last_span()), kind)))
    }

    fn expected() -> crate::Expected {
        crate::Expected::Nonterminal("expression")
    }
}

fn postfix_expr_kind<'db>(
    db: &'db dyn crate::Db,
    parser: &mut Parser<'_, 'db>,
) -> Result<Option<AstExprKind<'db>>, crate::ParseFail<'db>> {
    let start_span = parser.peek_span();

    let Some(mut kind) = base_expr_kind(db, parser)? else {
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
                let args = SquareBracketArgs::new(db, parser.last_span(), text.to_string());
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

fn base_expr_kind<'db>(
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
                let path = Path { ids: vec![id] };
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
