use crate::{token_test::*, tokens::Tokens};

use dada_ir::{
    diagnostic::Diagnostic, span::Span, token::Token, token_tree::TokenTree, word::Word,
};

mod code;
mod items;
pub(crate) struct Parser<'me> {
    db: &'me dyn crate::Db,
    tokens: Tokens<'me>,
    errors: &'me mut Vec<Diagnostic>,
}

impl<'me> Parser<'me> {
    pub(crate) fn new(
        db: &'me dyn crate::Db,
        tokens: Tokens<'me>,
        errors: &'me mut Vec<Diagnostic>,
    ) -> Self {
        Self { db, tokens, errors }
    }

    /// Returns Some if the next pending token matches `is`, along
    /// with the narrowed view of the next token.
    fn peek_if<TT: TokenTest>(&mut self, is: TT) -> Option<TT::Narrow> {
        is.test(self.db, &mut self.tokens)
    }

    /// If the next pending token matches `is`, consumes it and
    /// returns the span + narrowed view. Otherwise returns None
    /// and has no effect. Returns None if there is no pending token.
    fn eat_if<TT: TokenTest>(&mut self, is: TT) -> Option<(Span, TT::Narrow)> {
        let start_span = self.tokens.peek_span();
        let narrow = self.peek_if(is)?;
        self.tokens.consume();
        let end_span = self.tokens.last_span();
        Some((start_span.to(end_span), narrow))
    }

    /// If the next token is an opening delimiter, like `(` or `{`,
    /// then consumes it, the token-tree that follows, and the closing delimiter (if present).
    /// Returns the token tree + the span including delimiters.
    /// Reports an error if there is no closing delimiter.
    fn delimited(&mut self, delimiter: char) -> Option<(Span, TokenTree)> {
        let (open_span, _) = self.eat_if(Token::Delimiter(delimiter))?;

        // Lexer always produces a token tree as the next token after a delimiter:
        let (_, token_tree) = self.eat_if(AnyTree).unwrap();

        // Consume closing delimiter (if present)
        let closing_delimiter = dada_lex::closing_delimiter(delimiter);
        self.eat_if(Token::Delimiter(closing_delimiter))
            .or_report_error(self, || format!("expected `{closing_delimiter}`"));

        let span = open_span.to(self.tokens.last_span());
        Some((span, token_tree))
    }

    pub fn filename(&self) -> Word {
        self.tokens.filename()
    }

    /// Returns the span that starts at `span` and ends with the
    /// last consumed token.
    pub fn span_consumed_since(&self, span: Span) -> Span {
        span.to(self.tokens.last_span())
    }

    pub fn report_error_at_current_token(&mut self, message: impl AsRef<str>) {
        let span = self.tokens.peek_span();
        self.report_error(span, message)
    }

    pub fn report_error(&mut self, span: Span, message: impl AsRef<str>) {
        self.errors.push(Diagnostic {
            filename: self.filename(),
            span,
            message: message.as_ref().to_string(),
        });
    }
}

trait OrReportError {
    fn or_report_error(self, parser: &mut Parser<'_>, message: impl FnOnce() -> String) -> Self;
}

impl<T> OrReportError for Option<T> {
    fn or_report_error(self, parser: &mut Parser<'_>, message: impl FnOnce() -> String) -> Self {
        if self.is_some() {
            return self;
        }

        parser.report_error(parser.tokens.peek_span(), message());

        None
    }
}
