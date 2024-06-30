use salsa::update::Update;

use crate::{
    ast::{AstVec, Path},
    diagnostic::report_error,
};

use super::{
    tokenizer::{tokenize, Delimiter, Skipped, Token, TokenKind},
    OrNotPresent, ParseFail, ParseFailKind, ParseTokens, TokenStream,
};

impl<'db> ParseTokens<'db> for Path<'db> {
    fn parse(
        _db: &'db dyn crate::Db,
        tokens: &mut TokenStream<'_, 'db>,
    ) -> Result<Self, ParseFail<'db>> {
        let id = tokens.eat_spanned_id().or_not_present()?;
        let mut ids = vec![id];

        while tokens.eat_op('.').is_ok() {
            if let Ok(id) = tokens.eat_spanned_id() {
                ids.push(id);
            } else {
                break;
            }
        }

        Ok(Path { ids })
    }
}

/// Parses a comma-separated list of items.
/// Allows for trailing commas and newlines in place of commas.
pub fn parse_comma<'db, T>(
    db: &'db dyn crate::Db,
    tokens: &mut TokenStream<'_, 'db>,
    delimiter: Delimiter,
) -> Result<AstVec<'db, T>, ParseFail<'db>>
where
    T: ParseTokens<'db> + Update,
{
    let text = tokens.eat_delimited(delimiter).or_not_present()?;
    let text_span = tokens.last_span();
    let tokenized = tokenize(db, text_span.anchor, text_span.start, text);
    let tokens1 = &mut TokenStream::new(db, text_span.anchor, tokenized);

    let mut values = vec![];

    #[allow(unused_variables)]
    let uninitialized: String;

    loop {
        match T::parse(db, tokens1) {
            Ok(item) => {
                values.push(item);

                match tokens1.peek() {
                    // Consume a comma and keep parsing
                    Some(Token {
                        kind: TokenKind::OpChar(','),
                        ..
                    }) => {
                        tokens1.eat().unwrap();
                        continue;
                    }

                    // If the next token comes after a newline, parse it
                    Some(&Token { skipped, .. }) if skipped >= Some(Skipped::Newline) => continue,

                    // Anything else is an error
                    Some(token) => {
                        report_error(
                            db,
                            token.span,
                            "unexpected extra token after list item (missing comma or newline?)",
                        );
                        break;
                    }

                    // No more tokens? Great!
                    None => {
                        break;
                    }
                }
            }

            Err(ParseFail {
                span: _,
                kind: ParseFailKind::NotPresent,
            }) => break,

            Err(err) => {
                err.report(db);
                break;
            }
        }

        {
            #![allow(unreachable_code)]
            std::mem::drop(uninitialized);
        }
    }

    if let Some(token) = tokens1.peek() {
        report_error(db, token.span, "unexpected extra content");
    }

    Ok(AstVec {
        span: text_span,
        values,
    })
}
