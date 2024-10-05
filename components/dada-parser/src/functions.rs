use dada_ir_ast::ast::{
    AstBlock, AstExpr, AstFunction, AstFunctionInput, AstGenericDecl, AstLetStatement, AstPerm,
    AstSelfArg, AstStatement, AstTy, SpanVec, VariableDecl,
};

use crate::{
    miscellaneous::OrOptParse,
    tokenizer::{Delimiter, Keyword, Token, TokenKind},
    Expected, Parse, Parser,
};

impl<'db> Parse<'db> for AstFunction<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        tokens: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, super::ParseFail<'db>> {
        let start_span = tokens.peek_span();

        let Ok(fn_span) = tokens.eat_keyword(Keyword::Fn) else {
            return Ok(None);
        };

        let name = tokens.eat_id()?;

        let generics = AstGenericDecl::opt_parse_delimited(
            db,
            tokens,
            Delimiter::SquareBrackets,
            AstGenericDecl::eat_comma,
        )?;

        // Parse the arguments, accepting an empty list.
        let arguments = AstFunctionInput::eat_delimited(
            db,
            tokens,
            Delimiter::Parentheses,
            AstFunctionInput::opt_parse_comma,
        )?;
        let arguments = match arguments {
            Some(arguments) => arguments,
            None => SpanVec {
                span: tokens.last_span(),
                values: vec![],
            },
        };

        let return_ty = AstTy::opt_parse_guarded("->", db, tokens)?;

        let body = match tokens.defer_delimited(Delimiter::CurlyBraces) {
            Ok(b) => Some(b),
            Err(_) => None,
        };

        Ok(Some(AstFunction::new(
            db,
            start_span.to(tokens.last_span()),
            fn_span,
            name,
            generics,
            arguments,
            return_ty,
            body,
        )))
    }

    fn expected() -> Expected {
        Expected::Nonterminal("`fn`")
    }
}
impl<'db> Parse<'db> for AstFunctionInput<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, super::ParseFail<'db>> {
        if AstSelfArg::can_eat(db, parser) {
            Ok(Some(AstSelfArg::eat(db, parser)?.into()))
        } else if let Some(v) = VariableDecl::opt_parse(db, parser)? {
            Ok(Some(v.into()))
        } else {
            Ok(None)
        }
    }

    fn expected() -> Expected {
        Expected::Nonterminal("function argument")
    }
}

impl<'db> Parse<'db> for AstSelfArg<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, super::ParseFail<'db>> {
        // If we see a perm, this *must* be self...
        if let Some(perm) = AstPerm::opt_parse(db, parser)? {
            let self_var = parser.eat_id()?;
            if self_var.id.text(db) == "self" {
                Ok(Some(AstSelfArg::new(db, Some(perm), self_var.span)))
            } else {
                Err(parser.illformed(Self::expected()))
            }
        } else if let Some(&Token {
            kind: TokenKind::Identifier(id),
            span,
            ..
        }) = parser.peek()
        {
            // ...otherwise, it could be self...
            if id.text(db) == "self" {
                parser.eat_next_token()?;
                Ok(Some(AstSelfArg::new(db, None, span)))
            } else {
                Ok(None)
            }
        } else {
            // ...otherwise it ain't.
            Ok(None)
        }
    }

    fn expected() -> Expected {
        Expected::Nonterminal("self argument")
    }
}

#[salsa::tracked]
impl<'db> crate::prelude::FunctionBlock<'db> for AstFunction<'db> {
    #[salsa::tracked]
    fn body_block(self, db: &'db dyn crate::Db) -> Option<AstBlock<'db>> {
        let body = self.body(db).as_ref()?;
        Some(Parser::deferred(db, self, body, |parser| {
            let statements = parser.parse_many_and_report_diagnostics::<AstStatement>(db);
            AstBlock::new(db, statements)
        }))
    }
}

impl<'db> Parse<'db> for AstBlock<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self::Output>, crate::ParseFail<'db>> {
        let Some(statements) = AstStatement::opt_parse_delimited(
            db,
            parser,
            crate::tokenizer::Delimiter::CurlyBraces,
            AstStatement::eat_many,
        )?
        else {
            return Ok(None);
        };

        Ok(Some(AstBlock::new(db, statements)))
    }

    fn expected() -> crate::Expected {
        crate::Expected::Nonterminal("block")
    }
}

impl<'db> Parse<'db> for AstStatement<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self::Output>, crate::ParseFail<'db>> {
        AstLetStatement::opt_parse(db, parser).or_opt_parse::<Self, AstExpr>(db, parser)
    }

    fn expected() -> crate::Expected {
        crate::Expected::Nonterminal("statement")
    }
}

impl<'db> Parse<'db> for AstLetStatement<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self::Output>, crate::ParseFail<'db>> {
        let Ok(_) = parser.eat_keyword(Keyword::Let) else {
            return Ok(None);
        };
        let name = parser.eat_id()?;
        let ty = AstTy::opt_parse_guarded(":", db, parser)?;
        let initializer = AstExpr::opt_parse_guarded("=", db, parser)?;
        Ok(Some(AstLetStatement::new(db, name, ty, initializer)))
    }

    fn expected() -> crate::Expected {
        crate::Expected::Nonterminal("let statement")
    }
}
