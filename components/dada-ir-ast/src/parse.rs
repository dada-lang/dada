use std::iter::Peekable;

use tokenizer::{Delimiter, Keyword, Token, TokenKind};

use crate::{
    ast::{ClassItem, Identifier, Item, Module, Path, SpannedIdentifier, UseItem},
    diagnostic::report_error,
    inputs::SourceFile,
    span::{Offset, Span},
};

mod tokenizer;

pub struct TokenStream<'input, 'db> {
    tokens: Peekable<std::vec::IntoIter<Token<'input, 'db>>>,
    last_span: Span<'db>,
}

impl<'input, 'db> TokenStream<'input, 'db> {
    pub fn new(db: &'db dyn crate::Db, anchor: Item<'db>, tokens: Vec<Token<'input, 'db>>) -> Self {
        Self {
            tokens: tokens.into_iter().peekable(),
            last_span: anchor.span(db),
        }
    }

    pub fn peek(&mut self) -> Option<&Token<'input, 'db>> {
        self.tokens.peek()
    }

    pub fn last_span(&self) -> Span<'db> {
        self.last_span
    }

    pub fn parse_fail(&mut self, kind: ParseFailKind) -> ParseFail<'db> {
        let span = match self.peek() {
            Some(token) => token.span,
            None => self.last_span,
        };
        ParseFail { span, kind }
    }

    pub fn eat(&mut self) -> Result<(), ParseFail<'db>> {
        if let Some(token) = self.tokens.next() {
            self.last_span = token.span;
            Ok(())
        } else {
            Err(self.parse_fail(ParseFailKind::ExpectedToken))
        }
    }

    pub fn eat_keyword(&mut self, kw: Keyword) -> Result<Span<'db>, ParseFail<'db>> {
        if let Some(&Token {
            kind: TokenKind::Keyword(kw1),
            span,
        }) = self.peek()
        {
            if kw == kw1 {
                self.eat().unwrap();
                return Ok(span);
            }
        }
        Err(self.parse_fail(ParseFailKind::ExpectedKeyword(kw)))
    }

    pub fn eat_id(&mut self) -> Result<Identifier<'db>, ParseFail<'db>> {
        if let Some(&Token {
            kind: TokenKind::Identifier(id),
            span: _,
        }) = self.peek()
        {
            self.eat().unwrap();
            return Ok(id);
        }
        Err(self.parse_fail(ParseFailKind::ExpectedIdentifier))
    }

    pub fn eat_spanned_id(&mut self) -> Result<SpannedIdentifier<'db>, ParseFail<'db>> {
        if let Some(&Token {
            kind: TokenKind::Identifier(id),
            span,
        }) = self.peek()
        {
            self.eat().unwrap();
            return Ok(SpannedIdentifier { span, id });
        }
        Err(self.parse_fail(ParseFailKind::ExpectedIdentifier))
    }

    pub fn eat_op(&mut self, ch: char) -> Result<Span<'db>, ParseFail<'db>> {
        if let Some(&Token {
            kind: TokenKind::OpChar { ch: ch1, .. },
            span,
        }) = self.peek()
        {
            if ch == ch1 {
                self.eat().unwrap();
                return Ok(span);
            }
        }
        Err(self.parse_fail(ParseFailKind::ExpectedToken))
    }

    pub fn eat_delimited(&mut self, delimiter: Delimiter) -> Result<&'input str, ParseFail<'db>> {
        if let Some(&Token {
            kind:
                TokenKind::Delimited {
                    delimiter: delimiter1,
                    text,
                },
            span: _,
        }) = self.peek()
        {
            if delimiter == delimiter1 {
                self.eat().unwrap();
                return Ok(text);
            }
        }
        Err(self.parse_fail(ParseFailKind::ExpectedDelimited(delimiter)))
    }
}

pub trait ParseTokens<'db>: Sized {
    fn parse(
        db: &'db dyn crate::Db,
        tokens: &mut TokenStream<'_, 'db>,
    ) -> Result<Self, ParseFail<'db>>;
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct ParseFail<'db> {
    span: Span<'db>,
    kind: ParseFailKind,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ParseFailKind {
    NotPresent,
    ExpectedToken,
    ExpectedIdentifier,
    ExpectedKeyword(Keyword),
    ExpectedDelimited(Delimiter),
}

impl<'db> ParseFail<'db> {
    pub fn report(&self, db: &dyn crate::Db) {
        report_error(db, self.span, format!("parse failure: `{:?}`", self.kind))
    }
}

pub trait OrNotPresent {
    fn or_not_present(self) -> Self;
}

impl<'db, T> OrNotPresent for Result<T, ParseFail<'db>> {
    fn or_not_present(self) -> Self {
        match self {
            Ok(t) => Ok(t),
            Err(mut e) => {
                e.kind = ParseFailKind::NotPresent;
                Err(e)
            }
        }
    }
}

#[salsa::tracked]
impl SourceFile {
    pub fn parse<'db>(&self, db: &'db dyn crate::Db) -> Module<'db> {
        let anchor = Item::SourceFile(*self);
        let text = self.contents(db);
        let tokens = tokenizer::tokenize(db, anchor, Offset::ZERO, text);
        Module::parse(db, &mut TokenStream::new(db, anchor, tokens)).unwrap()
    }
}

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
            id,
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
