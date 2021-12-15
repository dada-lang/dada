use dada_ir::{span::Span, token::Token, token_tree::TokenTree, word::Word};

pub(crate) struct Tokens<'me> {
    db: &'me dyn crate::Db,
    filename: Word,

    /// Span of last token consumed.
    last_span: Span,
    skipped_newline: bool,
    tokens: &'me [Token],
}

impl<'me> Tokens<'me> {
    pub fn new(db: &'me dyn crate::Db, token_tree: TokenTree) -> Self {
        let tokens = token_tree.tokens(db);
        let filename = token_tree.filename(db);
        let mut this = Tokens {
            db,
            last_span: Span::start(),
            filename,
            tokens,
            skipped_newline: false,
        };
        this.skip_tokens();
        this
    }

    /// Returns the filename that these tokens are from
    pub fn filename(&self) -> Word {
        self.filename
    }

    fn next_token(&mut self) -> Option<Token> {
        if self.tokens.is_empty() {
            return None;
        }
        self.last_span = self.peek_span();
        self.tokens = &self.tokens[1..];
        self.peek()
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

            self.next_token();
        }
    }

    /// Advance by one token and return the span + token just consumed (if any).
    pub fn consume(&mut self) -> Option<(Span, Token)> {
        let token = self.next_token()?;
        let span = self.last_span;
        self.skipped_newline = false;

        self.skip_tokens();

        Some((span, token))
    }

    /// Span of the previously consumed token (or `Span::start` otherwise).
    pub fn last_span(&self) -> Span {
        self.last_span
    }

    /// Span of the next pending token (or last span if there are no more tokens).
    pub fn peek_span(&self) -> Span {
        match self.tokens.get(0) {
            None => self.last_span,
            Some(token) => {
                let len = token.span_len(self.db);
                let start = self.last_span.end;
                Span::from(start, start + len)
            }
        }
    }

    /// Next pending token, if any.
    pub fn peek(&self) -> Option<Token> {
        self.tokens.get(0).copied()
    }

    /// Peek ahead by n tokens; n == 0 is equivalent to `peek`.
    /// Use carefully as this exposes whitespace!
    pub fn peek_n(&self, n: usize) -> Option<Token> {
        self.tokens.get(n).copied()
    }
}
