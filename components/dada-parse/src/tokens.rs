use std::iter::Peekable;

use dada_ir::{span::Span, token::Token, token_tree::TokenTree, word::Word};

pub(crate) struct Tokens<'me> {
    filename: Word,
    last_span: Span,
    skipped_newline: bool,
    tokens: Peekable<Box<dyn Iterator<Item = (Span, Token)> + 'me>>,
}

impl<'me> Tokens<'me> {
    pub fn new(db: &'me dyn crate::Db, token_tree: TokenTree) -> Self {
        let tokens: Box<dyn Iterator<Item = (Span, Token)>> =
            Box::new(token_tree.spanned_tokens(db));
        let filename = token_tree.filename(db);
        Tokens {
            last_span: Span::start(),
            filename,
            tokens: tokens.peekable(),
            skipped_newline: false,
        }
    }

    /// Returns the filename that these tokens are from
    pub fn filename(&self) -> Word {
        self.filename
    }

    /// Skip tokens that the parser doesn't want to see,
    /// such as whitespace.
    fn skip_tokens(&mut self) {
        while let Some(t) = self.peek() {
            match t {
                Token::Whitespace('\n') => self.skipped_newline = true,
                Token::Whitespace(_) => (),
                _ => return,
            }

            self.tokens.next();
        }
    }

    /// Advance by one token and return the span + token just consumed (if any).
    pub fn consume(&mut self) -> Option<(Span, Token)> {
        let (span, token) = self.tokens.next()?;
        self.last_span = span;
        self.skipped_newline = false;

        self.skip_tokens();

        Some((span, token))
    }

    /// Span of the previously consumed token (or `Span::start` otherwise).
    pub fn last_span(&self) -> Span {
        self.last_span
    }

    /// Span of the next pending token (or last span if there are no more tokens).
    pub fn peek_span(&mut self) -> Span {
        match self.tokens.peek() {
            Some(pair) => pair.0,
            None => self.last_span,
        }
    }

    /// Next pending token, if any.
    pub fn peek(&mut self) -> Option<Token> {
        Some(self.tokens.peek()?.1)
    }
}
