use dada_ir_ast::{
    ast::{
        AstFieldDecl, AstFunction, AstFunctionArg, AstFunctionBody, AstGenericDecl, AstMember,
        AstPerm, AstSelfArg, AstTy, AstVisibility, AstClassItem, SpanVec, VariableDecl,
        VisibilityKind,
    },
    span::{Offset, Spanned},
};

use super::{
    miscellaneous::OrOptParse,
    tokenizer::{tokenize, Delimiter, Keyword, Token, TokenKind},
    Expected, Parse, Parser,
};

#[salsa::tracked]
impl<'db> crate::prelude::ClassItemMembers<'db> for AstClassItem<'db> {
    #[salsa::tracked]
    fn members(self, db: &'db dyn crate::Db) -> SpanVec<'db, AstMember<'db>> {
        let contents = self.contents(db);
        let tokens = tokenize(db, self.into(), Offset::ZERO, contents);
        Parser::new(db, self.into(), &tokens)
            .parse_many_and_report_diagnostics::<AstMember<'db>>(db)
    }
}

impl<'db> Parse<'db> for AstMember<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, super::ParseFail<'db>> {
        AstFieldDecl::opt_parse(db, parser).or_opt_parse::<Self, AstFunction<'db>>(db, parser)
    }

    fn expected() -> Expected {
        Expected::Nonterminal("class member")
    }
}

impl<'db> Parse<'db> for AstFieldDecl<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        tokens: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, super::ParseFail<'db>> {
        let visibility = AstVisibility::opt_parse(db, tokens)?;

        let variable = match VariableDecl::opt_parse(db, tokens) {
            Ok(Some(v)) => v,
            Ok(None) => {
                return if visibility.is_some() {
                    Err(tokens.illformed(VariableDecl::expected()))
                } else {
                    Ok(None)
                }
            }
            Err(e) => return Err(e),
        };

        let end_span = tokens.eat_op(";")?;

        Ok(Some(AstFieldDecl {
            span: visibility
                .as_ref()
                .map(|v| v.span)
                .unwrap_or_else(|| variable.span(db))
                .to(end_span),
            visibility,
            variable,
        }))
    }

    fn expected() -> Expected {
        Expected::Nonterminal("variable declaration")
    }
}

impl<'db> Parse<'db> for AstVisibility<'db> {
    type Output = Self;

    fn opt_parse(
        _db: &'db dyn crate::Db,
        tokens: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, super::ParseFail<'db>> {
        if let Ok(span) = tokens.eat_keyword(Keyword::Pub) {
            return Ok(Some(AstVisibility {
                span,
                kind: VisibilityKind::Pub,
            }));
        }

        if let Ok(span) = tokens.eat_keyword(Keyword::Export) {
            return Ok(Some(AstVisibility {
                span,
                kind: VisibilityKind::Export,
            }));
        }

        Ok(None)
    }

    fn expected() -> Expected {
        Expected::Nonterminal("visibility")
    }
}

impl<'db> Parse<'db> for VariableDecl<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        tokens: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, super::ParseFail<'db>> {
        let Ok(name) = tokens.eat_id() else {
            return Ok(None);
        };

        let _ = tokens.eat_op(":")?;

        let ty = AstTy::eat(db, tokens)?;

        Ok(Some(VariableDecl { name, ty }))
    }

    fn expected() -> Expected {
        Expected::Nonterminal("variable declaration")
    }
}

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
        let arguments = AstFunctionArg::eat_delimited(
            db,
            tokens,
            Delimiter::Parentheses,
            AstFunctionArg::opt_parse_comma,
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

impl<'db> Parse<'db> for AstFunctionArg<'db> {
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
                Ok(Some(AstSelfArg {
                    perm: Some(perm),
                    self_span: self_var.span,
                }))
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
                Ok(Some(AstSelfArg {
                    perm: None,
                    self_span: span,
                }))
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
