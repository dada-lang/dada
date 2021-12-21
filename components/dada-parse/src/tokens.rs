use dada_ir::{span::Span, token::Token, token_tree::TokenTree};

#[derive(Copy, Clone)]
pub(crate) struct Tokens<'me> {
    db: &'me dyn crate::Db,

    /// Span of last token consumed.
    last_span: Span,
    skipped: Skipped,
    tokens: &'me [Token],
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Skipped {
    None,
    Any,
    Newline,
}

impl<'me> Tokens<'me> {
    pub(crate) fn new(db: &'me dyn crate::Db, token_tree: TokenTree) -> Self {
        let start_span = token_tree.span(db).span_at_start();
        let tokens = token_tree.tokens(db);
        let mut this = Tokens {
            db,
            last_span: start_span,
            tokens,
            skipped: Skipped::None,
        };
        this.skip_tokens();
        this
    }

    fn next_token(&mut self) -> Option<Token> {
        if self.tokens.is_empty() {
            return None;
        }
        self.last_span = self.peek_span();
        let result = self.peek();
        self.tokens = &self.tokens[1..];

        result
    }

    /// True if we skipped a newline after consuming
    /// the last token.
    pub(crate) fn skipped_newline(&self) -> bool {
        self.skipped >= Skipped::Newline
    }

    /// True if we skipped whitespace after consuming
    /// the last token.
    pub(crate) fn skipped_any(&self) -> bool {
        self.skipped >= Skipped::Any
    }

    /// Skip tokens that the parser doesn't want to see,
    /// such as whitespace.
    fn skip_tokens(&mut self) {
        self.skipped = Skipped::None;
        while let Some(t) = self.peek() {
            match t {
                Token::Whitespace('\n') => self.skipped = self.skipped.max(Skipped::Newline),
                Token::Whitespace(_) => self.skipped = self.skipped.max(Skipped::Any),
                Token::Comment(_) => self.skipped = self.skipped.max(Skipped::Any),
                _ => return,
            }

            self.next_token();
        }
    }

    /// Advance by one token and return the span + token just consumed (if any).
    pub(crate) fn consume(&mut self) -> Option<Token> {
        let token = self.next_token()?;

        self.skip_tokens();

        Some(token)
    }

    /// Span of the previously consumed token (or `Span::start` otherwise).
    pub(crate) fn last_span(&self) -> Span {
        self.last_span
    }

    /// Span of the next pending token (or last span if there are no more tokens).
    pub(crate) fn peek_span(&self) -> Span {
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
    pub(crate) fn peek(&self) -> Option<Token> {
        self.tokens.get(0).copied()
    }
}
