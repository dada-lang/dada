use crate::ast::{AstGenericKind, GenericDecl, KindedGenericDecl};

use super::{
    tokenizer::{Keyword, Token, TokenKind},
    Expected, Parse, ParseFail, Parser,
};

impl<'db> Parse<'db> for GenericDecl<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, super::ParseFail<'db>> {
        let Some(kind) = AstGenericKind::opt_parse(db, parser)? else {
            return Ok(None);
        };

        let decl = KindedGenericDecl::eat(db, parser)?;
        Ok(Some(GenericDecl { kind, decl }))
    }

    fn expected() -> Expected {
        Expected::Nonterminal("generic declaration")
    }
}

impl<'db> Parse<'db> for AstGenericKind<'db> {
    type Output = AstGenericKind<'db>;

    fn opt_parse(
        _db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self::Output>, ParseFail<'db>> {
        match parser.peek() {
            Some(&Token {
                span,
                kind: TokenKind::Keyword(Keyword::Type),
                ..
            }) => Ok(Some(AstGenericKind::Type(span))),

            Some(&Token {
                span,
                kind: TokenKind::Keyword(Keyword::Perm),
                ..
            }) => Ok(Some(AstGenericKind::Perm(span))),

            _ => Ok(None),
        }
    }

    fn expected() -> Expected {
        todo!()
    }
}

impl<'db> Parse<'db> for KindedGenericDecl<'db> {
    type Output = Self;

    fn opt_parse(
        _db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self::Output>, ParseFail<'db>> {
        let Ok(name) = parser.eat_id() else {
            return Ok(None);
        };

        Ok(Some(KindedGenericDecl { name }))
    }

    fn expected() -> Expected {
        Expected::Nonterminal("name of generic parameter")
    }
}
