use dada_ir_ast::{
    ast::{
        AstBlock, AstExpr, AstFunction, AstFunctionEffects, AstFunctionInput, AstGenericDecl,
        AstLetStatement, AstPerm, AstSelfArg, AstStatement, AstTy, AstVisibility, SpanVec,
        VariableDecl,
    },
    diagnostic::{Diagnostic, Level},
    span::Span,
};
use salsa::Update;

use crate::{
    Expected, Parse, ParseFail, Parser,
    miscellaneous::OrOptParse,
    tokenizer::{Delimiter, Keyword, operator},
};

impl<'db> Parse<'db> for AstFunction<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, super::ParseFail<'db>> {
        if !AstFunctionPrefix::can_eat(db, parser) {
            return Ok(None);
        }

        let start_span = parser.peek_span();

        let AstFunctionPrefix {
            visibility,
            effects,
            fn_keyword: fn_span,
        } = AstFunctionPrefix::eat(db, parser)?;

        let name = parser.eat_id()?;

        let generics = AstGenericDecl::opt_parse_delimited(
            db,
            parser,
            Delimiter::SquareBrackets,
            AstGenericDecl::eat_comma,
        )?;

        // Parse the arguments, accepting an empty list.
        let arguments = AstFunctionInput::eat_delimited(
            db,
            parser,
            Delimiter::Parentheses,
            AstFunctionInput::opt_parse_comma,
        )?;
        let arguments = match arguments {
            Some(arguments) => arguments,
            None => SpanVec {
                span: parser.last_span(),
                values: vec![],
            },
        };

        let return_ty = AstTy::opt_parse_guarded(operator::ARROW, db, parser)?;

        let body = parser.defer_delimited(Delimiter::CurlyBraces).ok();

        Ok(Some(AstFunction::new(
            db,
            start_span.to(db, parser.last_span()),
            effects,
            fn_span,
            visibility,
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

/// The *prefix* parses a fn declaration up until
/// the `fn` keyword. That is what we need to see
/// to know that we should be parsing a function.
/// Parsing always succeeds with `Ok(Some)` or errors;
/// the intent is that you probe with `can_eat`.
#[derive(Update)]
struct AstFunctionPrefix<'db> {
    /// Visibility of the class
    visibility: Option<AstVisibility<'db>>,
    effects: AstFunctionEffects<'db>,
    fn_keyword: Span<'db>,
}

impl<'db> Parse<'db> for AstFunctionPrefix<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, ParseFail<'db>> {
        Ok(Some(AstFunctionPrefix {
            visibility: AstVisibility::opt_parse(db, parser)?,
            effects: AstFunctionEffects::eat(db, parser)?,
            fn_keyword: parser.eat_keyword(Keyword::Fn)?,
        }))
    }

    fn expected() -> Expected {
        Expected::Nonterminal("fn")
    }
}

impl<'db> Parse<'db> for AstFunctionEffects<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, super::ParseFail<'db>> {
        let mut effects = AstFunctionEffects::default();

        loop {
            if let Ok(span) = parser.eat_keyword(Keyword::Async) {
                if let Some(prev_span) = effects.async_effect {
                    report_duplicate_keyword(db, "async", span, prev_span);
                }
                effects.async_effect = Some(span);
                continue;
            }

            if let Ok(span) = parser.eat_keyword(Keyword::Unsafe) {
                if let Some(prev_span) = effects.unsafe_effect {
                    report_duplicate_keyword(db, "unsafe", span, prev_span);
                }
                effects.unsafe_effect = Some(span);
                continue;
            }

            break;
        }

        Ok(Some(effects))
    }

    fn expected() -> Expected {
        Expected::Nonterminal("function effects")
    }
}

fn report_duplicate_keyword<'db>(
    db: &'db dyn crate::Db,
    kw: &str,
    span: Span<'db>,
    prev_span: Span<'db>,
) {
    Diagnostic::error(db, span, format!("duplicate `{kw}` keyword"))
        .label(
            db,
            Level::Error,
            span,
            format!("`{kw}` keyword already specified"),
        )
        .label(
            db,
            Level::Error,
            prev_span,
            format!("previous `{kw}` keyword"),
        )
        .report(db);
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
            let self_span = parser.eat_keyword(Keyword::Self_)?;
            Ok(Some(AstSelfArg::new(db, perm, self_span)))
        } else if let Ok(span) = parser.eat_keyword(Keyword::Self_) {
            // ...otherwise, it could be self...
            let perm = AstPerm::new(db, span, dada_ir_ast::ast::AstPermKind::ImplicitShared);
            Ok(Some(AstSelfArg::new(db, perm, span)))
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
        let mutable = parser.eat_keyword(Keyword::Mut).ok();
        let name = parser.eat_id()?;
        let ty = AstTy::opt_parse_guarded(operator::COLON, db, parser)?;
        let initializer = AstExpr::opt_parse_guarded(operator::EQ, db, parser)?;
        Ok(Some(AstLetStatement::new(
            db,
            mutable,
            name,
            ty,
            initializer,
        )))
    }

    fn expected() -> crate::Expected {
        crate::Expected::Nonterminal("let statement")
    }
}
