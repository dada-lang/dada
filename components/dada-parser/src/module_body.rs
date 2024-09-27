use dada_ir_ast::{
    ast::{AstClassItem, AstFunction, AstItem, AstModule, AstPath, AstUseItem, SpanVec},
    diagnostic::Diagnostic,
};

use super::{
    miscellaneous::OrOptParse,
    tokenizer::{Delimiter, Keyword},
    Expected, Parse, ParseFail, Parser,
};

impl<'db> Parse<'db> for AstModule<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, ParseFail<'db>> {
        let mut items: Vec<AstItem<'db>> = vec![];

        // Parse items, skipping unrecognized tokens.
        let start_span = parser.peek_span();
        while let Some(token) = parser.peek() {
            let span = token.span;
            match AstItem::opt_parse(db, parser) {
                Ok(Some(v)) => items.push(v),
                Err(e) => parser.push_diagnostic(e.into_diagnostic(db)),
                Ok(None) => {
                    parser.eat_next_token().unwrap();
                    parser.push_diagnostic(Diagnostic::error(
                        db,
                        span,
                        "expected a module-level item",
                    ));
                }
            }
        }

        Ok(Some(AstModule::new(
            db,
            SpanVec {
                span: start_span.to(parser.last_span()),
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
        AstClassItem::opt_parse(db, parser)
            .or_opt_parse::<Self, AstUseItem<'db>>(db, parser)
            .or_opt_parse::<Self, AstFunction<'db>>(db, parser)
    }

    fn expected() -> Expected {
        panic!("module-level item (class, function, use)")
    }
}

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

        let body = parser.eat_delimited(Delimiter::CurlyBraces)?;

        Ok(Some(AstClassItem::new(
            db,
            start.to(parser.last_span()),
            id.id,
            id.span,
            body.to_string(),
        )))
    }

    fn expected() -> Expected {
        Expected::Keyword(Keyword::Class)
    }
}

/// use path [as name];
impl<'db> Parse<'db> for AstUseItem<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, ParseFail<'db>> {
        let Ok(start) = parser.eat_keyword(Keyword::Use) else {
            return Ok(None);
        };

        let crate_name = parser.eat_id()?;
        let _dot = parser.eat_op(".")?;
        let path = AstPath::eat(db, parser)?;

        let as_id = if parser.eat_keyword(Keyword::As).is_ok() {
            Some(parser.eat_id()?)
        } else {
            None
        };

        parser.eat_op(";")?;

        Ok(Some(AstUseItem::new(
            db,
            start.to(parser.last_span()),
            crate_name,
            path,
            as_id,
        )))
    }

    fn expected() -> Expected {
        Expected::Keyword(Keyword::Use)
    }
}
