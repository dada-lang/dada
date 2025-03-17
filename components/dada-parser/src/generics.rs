use dada_ir_ast::ast::{AstGenericDecl, AstGenericKind};

use super::{Expected, Parse, ParseFail, Parser, tokenizer::Keyword};

impl<'db> Parse<'db> for AstGenericDecl<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, super::ParseFail<'db>> {
        let Some(kind) = AstGenericKind::opt_parse(db, parser)? else {
            return Ok(None);
        };

        let decl = parser.eat_id().ok();
        Ok(Some(AstGenericDecl::new(db, kind, decl)))
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
        if let Ok(span) = parser.eat_keyword(Keyword::Type) {
            Ok(Some(AstGenericKind::Type(span)))
        } else if let Ok(span) = parser.eat_keyword(Keyword::Perm) {
            Ok(Some(AstGenericKind::Perm(span)))
        } else {
            Ok(None)
        }
    }

    fn expected() -> Expected {
        Expected::Nonterminal("`type` or `perm`")
    }
}
