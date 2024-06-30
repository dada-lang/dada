use crate::{
    ast::{ClassItem, Item, Module, Path, UseItem},
    diagnostic::report_error,
};

use super::{
    tokenizer::{self, Delimiter, Keyword},
    OrNotPresent, ParseFail, ParseTokens, TokenStream,
};

impl<'db> ParseTokens<'db> for Module<'db> {
    fn parse(
        db: &'db dyn crate::Db,
        tokens: &mut TokenStream<'_, 'db>,
    ) -> Result<Self, ParseFail<'db>> {
        let mut items: Vec<Item<'db>> = vec![];

        while let Some(token) = tokens.peek() {
            match token.kind {
                tokenizer::TokenKind::Keyword(Keyword::Class) => {
                    match ClassItem::parse(db, tokens) {
                        Ok(i) => items.push(i.into()),
                        Err(e) => e.report(db),
                    }
                }

                tokenizer::TokenKind::Keyword(Keyword::Use) => match UseItem::parse(db, tokens) {
                    Ok(i) => items.push(i.into()),
                    Err(e) => e.report(db),
                },

                _ => report_error(db, token.span, "unexpected token".to_string()),
            }
        }

        Ok(Module::new(db, items))
    }
}

/// class Name { ... }
impl<'db> ParseTokens<'db> for ClassItem<'db> {
    fn parse(
        db: &'db dyn crate::Db,
        tokens: &mut TokenStream<'_, 'db>,
    ) -> Result<Self, ParseFail<'db>> {
        let start = tokens.eat_keyword(Keyword::Class).or_not_present()?;

        let id = tokens.eat_id()?;

        let body = tokens.eat_delimited(Delimiter::CurlyBraces)?;

        Ok(ClassItem::new(
            db,
            start.to(tokens.last_span()),
            id.id,
            id.span,
            body.to_string(),
        ))
    }
}

/// use path [as name];
impl<'db> ParseTokens<'db> for UseItem<'db> {
    fn parse(
        db: &'db dyn crate::Db,
        tokens: &mut TokenStream<'_, 'db>,
    ) -> Result<Self, ParseFail<'db>> {
        let start = tokens.eat_keyword(Keyword::Use).or_not_present()?;

        let path = Path::parse(db, tokens)?;

        let opt_name = if tokens.eat_keyword(Keyword::As).is_ok() {
            Some(tokens.eat_id()?)
        } else {
            None
        };

        tokens.eat_op(';')?;

        Ok(UseItem::new(
            db,
            start.to(tokens.last_span()),
            path,
            opt_name,
        ))
    }
}
