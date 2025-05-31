use dada_ir_ast::{
    ast::{
        AstAggregate, AstFunction, AstItem, AstMainFunction, AstModule, AstPath, AstStatement,
        AstUse, SpanVec,
    },
    diagnostic::Diagnostic,
    span::Spanned,
};

use crate::tokenizer::operator;

use super::{Expected, Parse, ParseFail, Parser, miscellaneous::OrOptParse, tokenizer::Keyword};

impl<'db> Parse<'db> for AstModule<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, ParseFail<'db>> {
        // Derive the name of the module from the source file in the span.
        let name = parser.last_span().source_file(db).module_name(db);

        // Parse (item* statement*), skipping unrecognized tokens.
        let mut items: Vec<AstItem<'db>> = vec![];
        let mut statements = vec![];
        let start_span = parser.peek_span();
        while let Some(_) = parser.peek() {
            if statements.is_empty() {
                match AstItem::opt_parse(db, parser) {
                    Ok(Some(v)) => {
                        items.push(v);
                        continue;
                    }

                    Err(e) => {
                        parser.push_diagnostic(e.into_diagnostic(db));
                        continue;
                    }

                    Ok(None) => {}
                }
            }

            match AstStatement::opt_parse(db, parser) {
                Ok(Some(s)) => {
                    statements.push(s);
                    continue;
                }

                Err(e) => {
                    parser.push_diagnostic(e.into_diagnostic(db));
                    continue;
                }

                Ok(None) => {}
            }

            parser.eat_next_token().unwrap();
            parser.push_diagnostic(Diagnostic::error(
                db,
                parser.last_span(),
                "expected a statement or a module-level item",
            ));
        }

        // If we have statements on their own, wrap them in a `main` function
        if let Some(first) = statements.first()
            && let Some(last) = statements.last()
        {
            let span = first.span(db).to(db, last.span(db));
            let main_fn = AstMainFunction::new(
                db,
                SpanVec {
                    span,
                    values: statements,
                },
            );
            items.push(main_fn.into());
        }

        Ok(Some(AstModule::new(
            db,
            name,
            SpanVec {
                span: start_span.to(db, parser.last_span()),
                values: items,
            },
        )))
    }

    fn expected() -> Expected {
        panic!("infallible")
    }
}

impl<'db> Parse<'db> for AstItem<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, ParseFail<'db>> {
        AstAggregate::opt_parse(db, parser)
            .or_opt_parse::<Self, AstUse<'db>>(db, parser)
            .or_opt_parse::<Self, AstFunction<'db>>(db, parser)
    }

    fn expected() -> Expected {
        panic!("module-level item (class, function, use)")
    }
}

/// use path [as name];
impl<'db> Parse<'db> for AstUse<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, ParseFail<'db>> {
        let Ok(start) = parser.eat_keyword(Keyword::Use) else {
            return Ok(None);
        };

        let crate_name = parser.eat_id()?;
        let _dot = parser.eat_op(operator::DOT)?;
        let path = AstPath::eat(db, parser)?;

        let as_id = if parser.eat_keyword(Keyword::As).is_ok() {
            Some(parser.eat_id()?)
        } else {
            None
        };

        Ok(Some(AstUse::new(
            db,
            start.to(db, parser.last_span()),
            crate_name,
            path,
            as_id,
        )))
    }

    fn expected() -> Expected {
        Expected::Keyword(Keyword::Use)
    }
}
