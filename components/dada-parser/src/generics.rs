use dada_ir_ast::ast::{
    AstGenericDecl, AstGenericKind, AstGenericTerm, AstWhereClause, AstWhereClauseKind,
    AstWhereClauses, SpanVec,
};

use crate::tokenizer::operator;

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

impl<'db> Parse<'db> for AstWhereClauses<'db> {
    type Output = AstWhereClauses<'db>;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self::Output>, ParseFail<'db>> {
        let Ok(where_span) = parser.eat_keyword(Keyword::Where) else {
            return Ok(None);
        };
        let clauses = match AstWhereClause::opt_parse_comma(db, parser)? {
            Some(v) => v,
            None => SpanVec {
                span: where_span.to(db, parser.last_span()),
                values: vec![],
            },
        };
        Ok(Some(AstWhereClauses::new(db, where_span, clauses)))
    }

    fn expected() -> Expected {
        Expected::Nonterminal("`where`")
    }
}

impl<'db> Parse<'db> for AstWhereClause<'db> {
    type Output = AstWhereClause<'db>;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self::Output>, ParseFail<'db>> {
        let Some(subject) = AstGenericTerm::opt_parse(db, parser)? else {
            return Ok(None);
        };
        let Ok(is_span) = parser.eat_keyword(Keyword::Is) else {
            return Err(parser.illformed(Expected::Keyword(Keyword::Is)));
        };
        // Question: `where A is (shared, copy)` or `where A is shared + copy` or `where A is shared & copy`?
        let kinds = match AstWhereClauseKind::opt_parse_separated(db, parser, operator::PLUS)? {
            Some(v) => v,
            None => SpanVec {
                span: is_span.to(db, parser.last_span()),
                values: vec![],
            },
        };
        Ok(Some(AstWhereClause::new(db, subject, kinds)))
    }

    fn expected() -> Expected {
        Expected::Nonterminal("where clause")
    }
}

impl<'db> Parse<'db> for AstWhereClauseKind<'db> {
    type Output = AstWhereClauseKind<'db>;

    fn opt_parse(
        _db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self::Output>, ParseFail<'db>> {
        if let Ok(span) = parser.eat_keyword(Keyword::Ref) {
            Ok(Some(AstWhereClauseKind::Reference(span)))
        } else if let Ok(span) = parser.eat_keyword(Keyword::Mut) {
            Ok(Some(AstWhereClauseKind::Mutable(span)))
        } else if let Ok(span) = parser.eat_keyword(Keyword::Shared) {
            Ok(Some(AstWhereClauseKind::Shared(span)))
        } else if let Ok(span) = parser.eat_keyword(Keyword::Owned) {
            Ok(Some(AstWhereClauseKind::Owned(span)))
        } else if let Ok(span) = parser.eat_keyword(Keyword::Unique) {
            Ok(Some(AstWhereClauseKind::Unique(span)))
        } else if let Ok(span) = parser.eat_keyword(Keyword::Lent) {
            Ok(Some(AstWhereClauseKind::Lent(span)))
        } else {
            Ok(None)
        }
    }

    fn expected() -> Expected {
        Expected::Nonterminal("`where`")
    }
}
