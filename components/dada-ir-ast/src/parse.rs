use std::iter::Peekable;

use tokenizer::{Delimiter, Keyword, Token, TokenKind};

use crate::{
    ast::{Identifier, Item, Module, SpannedIdentifier},
    diagnostic::report_error,
    inputs::SourceFile,
    span::{Offset, Span},
};

mod class_body;
mod miscellaneous;
mod module_body;
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
            skipped: _,
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

    pub fn eat_id(&mut self) -> Result<SpannedIdentifier<'db>, ParseFail<'db>> {
        if let Some(&Token {
            kind: TokenKind::Identifier(id),
            span,
            skipped: _,
        }) = self.peek()
        {
            self.eat().unwrap();
            return Ok(SpannedIdentifier { span, id });
        }
        Err(self.parse_fail(ParseFailKind::ExpectedIdentifier))
    }

    pub fn eat_spanned_id(&mut self) -> Result<SpannedIdentifier<'db>, ParseFail<'db>> {
        if let Some(&Token {
            kind: TokenKind::Identifier(id),
            span,
            skipped: _,
        }) = self.peek()
        {
            self.eat().unwrap();
            return Ok(SpannedIdentifier { span, id });
        }
        Err(self.parse_fail(ParseFailKind::ExpectedIdentifier))
    }

    pub fn eat_op(&mut self, ch: char) -> Result<Span<'db>, ParseFail<'db>> {
        if let Some(&Token {
            kind: TokenKind::OpChar(ch1),
            span,
            skipped: _,
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
            skipped: _,
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
