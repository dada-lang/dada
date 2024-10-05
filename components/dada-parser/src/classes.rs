use dada_ir_ast::{
    ast::{
        AstClassItem, AstFieldDecl, AstFunction, AstGenericDecl, AstMember, AstTy, AstVisibility,
        SpanVec, VariableDecl, VisibilityKind,
    },
    span::Spanned,
};

use crate::ParseFail;

use super::{
    miscellaneous::OrOptParse,
    tokenizer::{Delimiter, Keyword},
    Expected, Parse, Parser,
};

/// class Name { ... }
impl<'db> Parse<'db> for AstClassItem<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, ParseFail<'db>> {
        let Ok(start) = parser.eat_keyword(Keyword::Class) else {
            return Ok(None);
        };

        let id = parser.eat_id()?;

        let generics = AstGenericDecl::opt_parse_delimited(
            db,
            parser,
            Delimiter::SquareBrackets,
            AstGenericDecl::eat_comma,
        )?;

        let body = parser.defer_delimited(Delimiter::CurlyBraces)?;

        Ok(Some(AstClassItem::new(
            db,
            start.to(parser.last_span()),
            id.id,
            id.span,
            generics,
            body,
        )))
    }

    fn expected() -> Expected {
        Expected::Keyword(Keyword::Class)
    }
}

#[salsa::tracked]
impl<'db> crate::prelude::ClassItemMembers<'db> for AstClassItem<'db> {
    #[salsa::tracked]
    fn members(self, db: &'db dyn crate::Db) -> SpanVec<'db, AstMember<'db>> {
        let contents = self.contents(db);
        Parser::deferred(db, self, contents, |parser| {
            parser.parse_many_and_report_diagnostics::<AstMember<'db>>(db)
        })
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

        Ok(Some(AstFieldDecl::new(
            db,
            visibility
                .as_ref()
                .map(|v| v.span)
                .unwrap_or_else(|| variable.span(db))
                .to(variable.span(db)),
            visibility,
            variable,
        )))
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

        Ok(Some(VariableDecl::new(db, name, ty)))
    }

    fn expected() -> Expected {
        Expected::Nonterminal("variable declaration")
    }
}
