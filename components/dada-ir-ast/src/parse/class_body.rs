use crate::{
    ast::{
        AstFunctionArg, AstPerm, AstSelfArg, AstTy, AstVec, ClassItem, FieldDecl, Function,
        FunctionBody, GenericDecl, Item, Member, VariableDecl, Visibility, VisibilityKind,
    },
    span::{Offset, Spanned},
};

use super::{
    miscellaneous::OrOptParse,
    tokenizer::{tokenize, Delimiter, Keyword, Token, TokenKind},
    Expected, Parse, Parser,
};

#[salsa::tracked]
impl<'db> ClassItem<'db> {
    pub fn members(&self, db: &'db dyn crate::Db) -> AstVec<'db, Member<'db>> {
        let contents = self.contents(db);
        let tokens = tokenize(db, Item::from(*self), Offset::ZERO, contents);
        Parser::new(db, Item::Class(*self), &tokens)
            .parse_many_and_report_diagnostics::<Member<'db>>(db)
    }
}

impl<'db> Parse<'db> for Member<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, super::ParseFail<'db>> {
        FieldDecl::opt_parse(db, parser).or_opt_parse::<Self, Function<'db>>(db, parser)
    }

    fn expected() -> Expected {
        Expected::Nonterminal("class member")
    }
}

impl<'db> Parse<'db> for FieldDecl<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        tokens: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, super::ParseFail<'db>> {
        let visibility = Visibility::opt_parse(db, tokens)?;

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

        Ok(Some(FieldDecl {
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

impl<'db> Parse<'db> for Visibility<'db> {
    type Output = Self;

    fn opt_parse(
        _db: &'db dyn crate::Db,
        tokens: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, super::ParseFail<'db>> {
        if let Ok(span) = tokens.eat_keyword(Keyword::Pub) {
            return Ok(Some(Visibility {
                span,
                kind: VisibilityKind::Pub,
            }));
        }

        if let Ok(span) = tokens.eat_keyword(Keyword::Export) {
            return Ok(Some(Visibility {
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

impl<'db> Parse<'db> for Function<'db> {
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

        let generics = GenericDecl::opt_parse_delimited(db, tokens, Delimiter::SquareBrackets)?;

        let arguments = AstFunctionArg::eat_delimited(db, tokens, Delimiter::Parentheses)?;

        let return_ty = match tokens.eat_op("->") {
            Ok(_) => Some(AstTy::eat(db, tokens)?),
            Err(_) => None,
        };

        let body = match tokens.eat_op(";") {
            Ok(_) => None,
            Err(_) => Some(FunctionBody::eat(db, tokens)?),
        };

        Ok(Some(Function::new(
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
        AstSelfArg::opt_parse(db, parser).or_opt_parse::<Self, VariableDecl<'db>>(db, parser)
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

impl<'db> Parse<'db> for FunctionBody<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self::Output>, super::ParseFail<'db>> {
        let Ok(text) = parser.eat_delimited(Delimiter::CurlyBraces) else {
            return Ok(None);
        };

        Ok(Some(FunctionBody::new(
            db,
            parser.last_span(),
            text.to_string(),
        )))
    }

    fn expected() -> Expected {
        Expected::Nonterminal("function body")
    }
}
