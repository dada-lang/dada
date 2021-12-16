use crate::{token_test::*, tokens::Tokens};

use dada_ir::{
    diagnostic::Diagnostic, op::Op, span::Span, token::Token, token_tree::TokenTree, word::Word,
};

mod code;
mod items;
pub(crate) struct Parser<'me> {
    db: &'me dyn crate::Db,
    filename: Word,
    tokens: Tokens<'me>,
    errors: &'me mut Vec<Diagnostic>,
}

impl<'me> Parser<'me> {
    pub(crate) fn new(
        db: &'me dyn crate::Db,
        token_tree: TokenTree,
        errors: &'me mut Vec<Diagnostic>,
    ) -> Self {
        let tokens = Tokens::new(db, token_tree);
        let filename = token_tree.filename(db);
        Self {
            db,
            tokens,
            filename,
            errors,
        }
    }

    /// Returns Some if the next pending token matches `is`, along
    /// with the narrowed view of the next token.
    fn peek_if<TT: TokenTest>(&mut self, is: TT) -> Option<TT::Narrow> {
        is.test(self.db, self.tokens.peek()?)
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

    /// Peek ahead to see if `op` matches the next set of tokens;
    /// if so, return the span and the tokens after skipping the operator.
    fn test_op(&self, op: Op) -> Option<(Span, Tokens<'me>)> {
        let mut tokens = self.tokens;
        let span0 = tokens.last_span();

        for ch in op.str().chars() {
            match tokens.consume() {
                Some(Token::Op(ch1)) if ch == ch1 => continue,
                _ => return None,
            }
        }

        let span = span0.to(tokens.last_span());

        // Careful: for most operators, if we are looking for `+`
        // and we see `++` or `+-` or something like that,
        // we don't want that to match!
        if Op::ACCEPT_ADJACENT.contains(&op) {
            Some((span, tokens))
        } else {
            match tokens.consume() {
                Some(Token::Op(_)) => None,
                _ => Some((span, tokens)),
            }
        }
    }

    /// If the next tokens match the given operator, consume it and
    /// return.
    fn eat_op(&mut self, op: Op) -> Option<Span> {
        let (span, tokens) = self.test_op(op)?;
        self.tokens = tokens;
        Some(span)
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
        self.filename
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

    pub fn report_error_if_more_tokens(&mut self, message: impl AsRef<str>) {
        if self.tokens.peek().is_some() {
            self.report_error_at_current_token(message);
        }
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
