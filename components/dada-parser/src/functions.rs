use dada_ir_ast::{
    ast::{
        AstBlock, AstExpr, AstFunction, AstFunctionBody, AstFunctionInput, AstGenericDecl,
        AstLetStatement, AstPerm, AstSelfArg, AstStatement, AstTy, SpanVec, VariableDecl,
    },
    span::Offset,
};

use crate::{
    miscellaneous::OrOptParse,
    tokenizer::{tokenize, Delimiter, Keyword, Token, TokenKind},
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

        let body = match tokens.eat_op(";") {
            Ok(_) => None,
            Err(_) => Some(AstFunctionBody::eat(db, tokens)?),
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

impl<'db> Parse<'db> for AstFunctionBody<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self::Output>, super::ParseFail<'db>> {
        let Ok(text) = parser.eat_delimited(Delimiter::CurlyBraces) else {
            return Ok(None);
        };

        Ok(Some(AstFunctionBody::new(
            db,
            parser.last_span(),
            text.to_string(),
        )))
    }

    fn expected() -> Expected {
        Expected::Nonterminal("function body")
    }
}

#[salsa::tracked]
impl<'db> crate::prelude::FunctionBodyBlock<'db> for AstFunctionBody<'db> {
    #[salsa::tracked]
    fn block(self, db: &'db dyn crate::Db) -> AstBlock<'db> {
        let contents = self.contents(db);
        let tokens = tokenize(db, self.into(), Offset::ZERO, contents);
        let statements = Parser::new(db, self.into(), &tokens)
            .parse_many_and_report_diagnostics::<AstStatement>(db);
        AstBlock::new(db, statements)
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
