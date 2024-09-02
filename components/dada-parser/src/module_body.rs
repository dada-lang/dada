use dada_ir_ast::{
    ast::{AstVec, ClassItem, Function, Item, Module, Path, UseItem},
    diagnostic::Diagnostic,
};

use super::{
    miscellaneous::OrOptParse,
    tokenizer::{Delimiter, Keyword},
    Expected, Parse, ParseFail, Parser,
};

impl<'db> Parse<'db> for Module<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, ParseFail<'db>> {
        let mut items: Vec<Item<'db>> = vec![];

        // Parse items, skipping unrecognized tokens.
        let start_span = parser.peek_span();
        while let Some(token) = parser.peek() {
            let span = token.span;
            match Item::opt_parse(db, parser) {
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

        Ok(Some(Module::new(
            db,
            AstVec {
                span: start_span.to(parser.last_span()),
                values: items,
            },
        )))
    }

    fn expected() -> Expected {
        panic!("infallible")
    }
}

impl<'db> Parse<'db> for Item<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, ParseFail<'db>> {
        ClassItem::opt_parse(db, parser)
            .or_opt_parse::<Self, UseItem<'db>>(db, parser)
            .or_opt_parse::<Self, Function<'db>>(db, parser)
    }

    fn expected() -> Expected {
        panic!("module-level item (class, function, use)")
    }
}

/// class Name { ... }
impl<'db> Parse<'db> for ClassItem<'db> {
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

        Ok(Some(ClassItem::new(
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
impl<'db> Parse<'db> for UseItem<'db> {
    type Output = Self;

    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self>, ParseFail<'db>> {
        let Ok(start) = parser.eat_keyword(Keyword::Use) else {
            return Ok(None);
        };

        let path = Path::eat(db, parser)?;

        let opt_name = if parser.eat_keyword(Keyword::As).is_ok() {
            Some(parser.eat_id()?)
        } else {
            None
        };

        parser.eat_op(";")?;

        Ok(Some(UseItem::new(
            db,
            start.to(parser.last_span()),
            path,
            opt_name,
        )))
    }

    fn expected() -> Expected {
        Expected::Keyword(Keyword::Use)
    }
}
