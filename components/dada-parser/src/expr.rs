use dada_ir_ast::ast::{
    AstCallExpr, AstConstructorField, AstExpr, AstExprKind, AstGenericArg, Literal, Path,
};

use crate::{
    tokenizer::{Keyword, TokenKind},
    Expected, Parse, Parser,
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

    let Some(base_kind) = base_expr_kind(db, parser)? else {
        return Ok(None);
    };

    let mid_span = parser.peek_span();

    if parser.next_token_on_same_line() {
        // Could be a call with generic args, like `foo.bar[T]()`
        let generic_args = AstGenericArg::opt_parse_delimited(
            db,
            parser,
            crate::tokenizer::Delimiter::SquareBrackets,
            AstGenericArg::eat_comma,
        )?;

        let args = AstExpr::opt_parse_delimited(
            db,
            parser,
            crate::tokenizer::Delimiter::Parentheses,
            AstExpr::eat_comma,
        )?;

        if let Some(args) = args {
            let callee = AstExpr::new(start_span.to(mid_span), base_kind);
            return Ok(Some(AstExprKind::Call(AstCallExpr {
                callee,
                generic_args,
                args,
            })));
        } else if let Some(_generic_args) = generic_args {
            // Can't have `foo.bar[X]` with no `()` afterwards.
            return Err(parser.illformed(Expected::Nonterminal("call arguments")));
        }
    }

    return Ok(Some(base_kind));
}

fn base_expr_kind<'db>(
    db: &'db dyn crate::Db,
    parser: &mut Parser<'_, 'db>,
) -> Result<Option<AstExprKind<'db>>, crate::ParseFail<'db>> {
    if let Some(literal) = Literal::opt_parse(db, parser)? {
        return Ok(Some(AstExprKind::Literal(literal)));
    }

    if let Some(path) = Path::opt_parse(db, parser)? {
        // Could be `X { field1: value1, .. }`
        if parser.next_token_on_same_line() {
            if let Some(fields) = AstConstructorField::opt_parse_delimited(
                db,
                parser,
                crate::tokenizer::Delimiter::CurlyBraces,
                AstConstructorField::eat_comma,
            )? {
                return Ok(Some(AstExprKind::Constructor(path, fields)));
            }
        }

        return Ok(Some(AstExprKind::Path(path)));
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
        let Some(next_token) = parser.peek() else {
            return Ok(None);
        };

        if let TokenKind::Literal(kind, text) = next_token.kind {
            Ok(Some(Literal::new(db, kind, text.to_string())))
        } else {
            Err(parser.illformed(Self::expected()))
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

        let _colon = parser.eat_op(":")?;

        let value = AstExpr::eat(db, parser)?;

        Ok(Some(AstConstructorField { name, value }))
    }

    fn expected() -> crate::Expected {
        crate::Expected::Nonterminal("field initializer")
    }
}
