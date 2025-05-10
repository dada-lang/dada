use dada_ir_ast::{
    ast::{
        AstAggregate, AstAggregateKind, AstFieldDecl, AstFunction, AstGenericDecl, AstMember,
        AstTy, AstTyKind, AstVisibility, SpanVec, VariableDecl, VisibilityKind,
    },
    span::{Span, Spanned},
};
use salsa::Update;

use crate::{ParseFail, tokenizer::operator};

use super::{
    Expected, Parse, Parser,
    miscellaneous::OrOptParse,
    tokenizer::{Delimiter, Keyword},
};

/// class Name { ... }
impl<'db> Parse<'db> for AstAggregate<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, ParseFail<'db>> {
        if !AstAggregatePrefix::can_eat(db, parser) {
            return Ok(None);
        }

        let start = parser.peek_span();

        let AstAggregatePrefix {
            visibility,
            aggregate_kind,
            aggregate_keyword: _,
        } = AstAggregatePrefix::eat(db, parser)?;

        let id = parser.eat_id()?;

        let generics = AstGenericDecl::opt_parse_delimited(
            db,
            parser,
            Delimiter::SquareBrackets,
            AstGenericDecl::eat_comma,
        )?;

        let inputs = AstFieldDecl::opt_parse_delimited(
            db,
            parser,
            Delimiter::Parentheses,
            AstFieldDecl::eat_comma,
        )?;

        let body = parser.defer_delimited(Delimiter::CurlyBraces).ok();

        Ok(Some(AstAggregate::new(
            db,
            start.to(db, parser.last_span()),
            visibility,
            aggregate_kind,
            id.id,
            id.span,
            generics,
            inputs,
            body,
        )))
    }

    fn expected() -> Expected {
        Expected::Keyword(Keyword::Class)
    }
}

/// The *prefix* parses a class declaration up until
/// the `class` keyword. That is what we need to see
/// to know that we should be parsing a class.
/// Parsing always succeeds with `Ok(Some)` or errors;
/// the intent is that you probe with `can_eat`.
#[derive(Update)]
struct AstAggregatePrefix<'db> {
    /// Visibility of the class
    visibility: Option<AstVisibility<'db>>,
    aggregate_kind: AstAggregateKind,
    aggregate_keyword: Span<'db>,
}

impl<'db> Parse<'db> for AstAggregatePrefix<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, ParseFail<'db>> {
        let visibility = AstVisibility::opt_parse(db, parser)?;

        if let Ok(span) = parser.eat_keyword(Keyword::Class) {
            Ok(Some(AstAggregatePrefix {
                visibility,
                aggregate_kind: AstAggregateKind::Class,
                aggregate_keyword: span,
            }))
        } else if let Ok(span) = parser.eat_keyword(Keyword::Struct) {
            Ok(Some(AstAggregatePrefix {
                visibility,
                aggregate_kind: AstAggregateKind::Struct,
                aggregate_keyword: span,
            }))
        } else {
            Ok(None)
        }
    }

    fn expected() -> Expected {
        Expected::Nonterminal("class")
    }
}

#[salsa::tracked]
impl<'db> crate::prelude::ClassItemMembers<'db> for AstAggregate<'db> {
    #[salsa::tracked(return_ref)]
    fn members(self, db: &'db dyn crate::Db) -> SpanVec<'db, AstMember<'db>> {
        if let Some(contents) = self.contents(db) {
            Parser::deferred(db, self, contents, |parser| {
                parser.parse_many_and_report_diagnostics::<AstMember<'db>>(db)
            })
        } else {
            SpanVec {
                span: self.span(db).at_end(),
                values: vec![],
            }
        }
    }
}

impl<'db> Parse<'db> for AstMember<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, super::ParseFail<'db>> {
        // Subtle: ordering is important here.
        // We prefer to parse members that have distinctive keywords
        // first (e.g., `fn`) because they are able to return `Ok(None)` when
        // that keyword is not present, allowing easier detection of which
        // form is correct. In principle we could modify `AstFieldDecl`'s parser
        // to fail more gracefully, but it's easier to just reorder things here.
        AstFunction::opt_parse(db, parser).or_opt_parse::<Self, AstFieldDecl<'db>>(db, parser)
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
                };
            }
            Err(e) => return Err(e),
        };

        Ok(Some(AstFieldDecl::new(
            db,
            visibility
                .as_ref()
                .map(|v| v.span)
                .unwrap_or_else(|| variable.span(db))
                .to(db, variable.span(db)),
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
        let (mutable, name);
        if let Ok(span) = tokens.eat_keyword(Keyword::Mut) {
            mutable = Some(span);
            name = tokens.eat_id()?;
        } else if let Ok(id) = tokens.eat_id() {
            mutable = None;
            name = id;
        } else {
            return Ok(None);
        };

        let _ = tokens.eat_op(operator::COLON)?;

        let ty = AstTy::eat(db, tokens)?;

        let (perm, base_ty) = match ty.kind(db) {
            AstTyKind::Perm(ast_perm, ast_ty) => (Some(ast_perm), ast_ty),
            AstTyKind::Named(..) => (None, ty),
            AstTyKind::GenericDecl(..) => (None, ty),
        };

        Ok(Some(VariableDecl::new(db, mutable, name, perm, base_ty)))
    }

    fn expected() -> Expected {
        Expected::Nonterminal("variable declaration")
    }
}
