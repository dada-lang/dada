use std::string::ToString;

use crate::{token_test::*, tokens::Tokens};

use dada_ir::{
    code::syntax::op::Op, diagnostic::DiagnosticBuilder, filename::Filename, span::Span,
    token::Token, token_tree::TokenTree,
};

mod code;
mod items;
mod parameter;
mod ty;

pub(crate) struct Parser<'me> {
    db: &'me dyn crate::Db,
    filename: Filename,
    tokens: Tokens<'me>,
}

impl<'me> Parser<'me> {
    pub(crate) fn new(db: &'me dyn crate::Db, token_tree: TokenTree) -> Self {
        let tokens = Tokens::new(db, token_tree);
        let filename = token_tree.filename(db);
        Self {
            db,
            tokens,
            filename,
        }
    }

    /// Returns `Some` if the next pending token matches `is`, along
    /// with the narrowed view of the next token.
    fn peek<TT: TokenTest>(&mut self, test: TT) -> Option<TT::Narrow> {
        let span = self.tokens.peek_span().in_file(self.filename);
        test.test(self.db, self.tokens.peek()?, span)
    }

    /// If the next pending token matches `test`, consumes it and
    /// returns the span + narrowed view. Otherwise returns None
    /// and has no effect. Returns None if there is no pending token.
    fn eat<TT: TokenTest>(&mut self, test: TT) -> Option<(Span, TT::Narrow)> {
        let span = self.tokens.peek_span();
        let narrow = self.peek(test)?;
        self.tokens.consume();
        Some((span, narrow))
    }

    /// Run `op` -- if it returns `None`, then no tokens are consumed.
    /// If it returns `Some`, then the tokens are consumed.
    /// Use sparingly, and try not to report errors or have side-effects in `op`.
    fn lookahead<R>(&mut self, op: impl FnOnce(&mut Self) -> Option<R>) -> Option<R> {
        let tokens = self.tokens;
        let r = op(self);
        if r.is_none() {
            // Restore tokens that `op` may have consumed.
            self.tokens = tokens;
        }
        r
    }

    /// Run `op` to get a true/false but rollback any tokens consumed.
    /// This is used to probe a few tokens ahead to see if we should
    /// commit to a given function.
    fn testahead(&mut self, op: impl FnOnce(&mut Self) -> bool) -> bool {
        let tokens = self.tokens;
        let r = op(self);
        self.tokens = tokens;
        r
    }

    /// Peek ahead to see if `op` matches the next set of tokens;
    /// if so, return the span and the tokens after skipping the operator.
    fn test_op(&self, op: Op) -> Option<(Span, Tokens<'me>)> {
        let mut tokens = self.tokens;
        let span0 = tokens.peek_span();

        let mut chars = op.str().chars();

        let ch0 = chars.next().unwrap();
        match tokens.consume() {
            Some(Token::Op(ch1)) if ch0 == ch1 => (),
            _ => return None,
        }

        for ch in chars {
            if tokens.skipped_any() {
                return None;
            }

            match tokens.consume() {
                Some(Token::Op(ch1)) if ch == ch1 => continue,
                _ => return None,
            }
        }

        let span = span0.to(tokens.last_span());

        // Careful: for most operators, if we are looking for `+`
        // and we see `++` or `+-` or something like that,
        // we don't want that to match!

        // If we skipped whitespace, then the token was on its own.
        if tokens.skipped_any() {
            return Some((span, tokens));
        }

        // For some operators, it doesn't matter if they are adjacent
        // to other operators.
        if Op::ACCEPT_ADJACENT.contains(&op) {
            return Some((span, tokens));
        }

        // Only return Some if this is the complete operator
        // (i.e., the operator `=` cannot match against a prefix of the input `==`)
        if let Some(Token::Op(_)) = tokens.peek() {
            return None;
        }

        // ...if not, we've got a match!
        Some((span, tokens))
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
        let (open_span, _) = self.eat(Token::Delimiter(delimiter))?;

        // Lexer always produces a token tree as the next token after a delimiter:
        let (_, token_tree) = self.eat(AnyTree).unwrap();

        // Consume closing delimiter (if present)
        let closing_delimiter = dada_lex::closing_delimiter(delimiter);
        self.eat(Token::Delimiter(closing_delimiter))
            .or_report_error(self, || format!("expected `{closing_delimiter}`"));

        let span = open_span.to(self.tokens.last_span());
        Some((span, token_tree))
    }

    /// Returns the span that starts at `span` and ends with the
    /// last consumed token.
    fn span_consumed_since(&self, span: Span) -> Span {
        self.tighten_span(span.to(self.tokens.last_span()))
    }

    fn tighten_span(&self, mut span: Span) -> Span {
        let strip_from_start = span
            .snippet(self.db, self.filename)
            .char_indices()
            .take_while(|(_, ch)| ch.is_whitespace())
            .map(|(offset, _)| offset)
            .last()
            .unwrap_or(0);
        span.start = span.start + strip_from_start;

        if let Some(new_len) = span
            .snippet(self.db, self.filename)
            .char_indices()
            .rev()
            .take_while(|(_, ch)| ch.is_whitespace())
            .map(|(offset, _)| offset)
            .last()
        {
            span.end = span.start + new_len;
        }

        span
    }

    fn emit_error_if_more_tokens(&self, message: impl ToString) {
        self.emit_labeled_error_if_more_tokens(message, |d| d)
    }

    fn emit_labeled_error_if_more_tokens(
        &self,
        message: impl ToString,
        op: impl FnOnce(DiagnosticBuilder) -> DiagnosticBuilder,
    ) {
        if self.tokens.peek().is_some() {
            op(self.error_at_current_token(message)).emit(self.db);
        }
    }

    fn error_at_current_token(&self, message: impl ToString) -> DiagnosticBuilder {
        let span = self.tokens.peek_span();
        self.error(span, message)
    }

    fn error(&self, span: Span, message: impl ToString) -> DiagnosticBuilder {
        dada_ir::error!(span.in_file(self.filename), "{}", message.to_string())
    }
}

trait OrReportError {
    fn or_report_error<S>(self, parser: &mut Parser<'_>, message: impl FnOnce() -> S) -> Self
    where
        S: ToString;

    fn or_report_error_at<S>(
        self,
        parser: &mut Parser<'_>,
        span: Span,
        message: impl FnOnce() -> S,
    ) -> Self
    where
        S: ToString;
}

impl<T> OrReportError for Option<T> {
    fn or_report_error<S>(self, parser: &mut Parser<'_>, message: impl FnOnce() -> S) -> Self
    where
        S: ToString,
    {
        self.or_report_error_at(parser, parser.tokens.peek_span(), message)
    }

    fn or_report_error_at<S>(
        self,
        parser: &mut Parser<'_>,
        span: Span,
        message: impl FnOnce() -> S,
    ) -> Self
    where
        S: ToString,
    {
        if self.is_some() {
            return self;
        }

        parser.error(span, message()).emit(parser.db);

        None
    }
}

trait ParseList {
    /// Parses a list of items. The items must be separated by either the given separator `sep` (if any)
    /// or a newline. Trailing separators are ok. For example, given given `sep = Op::Comma`, any of the following are accepted:
    ///
    /// * `foo, bar`
    /// * `foo, bar,`
    /// * `foo \n bar`
    /// * `foo, \n bar`
    /// * `foo, \n bar,`
    ///
    /// The following is not accepted:
    ///
    /// * `foo bar`
    #[tracing::instrument(level = "debug", skip(self, parse_item))]
    fn parse_list<T>(
        &mut self,
        comma_separated: bool,
        mut parse_item: impl FnMut(&mut Self) -> Option<T>,
    ) -> Vec<T> {
        let mut v = vec![];
        while let Some(i) = parse_item(self) {
            v.push(i);

            // List items can always be separated by a newline...
            if !self.skipped_newline() {
                // ...otherwise, they *may* require a separator
                if comma_separated && !self.eat_comma() {
                    break;
                }
            }
        }
        v.shrink_to_fit();
        v
    }

    fn skipped_newline(&self) -> bool;
    fn eat_comma(&mut self) -> bool;
}

impl ParseList for Parser<'_> {
    fn skipped_newline(&self) -> bool {
        self.tokens.skipped_newline()
    }

    fn eat_comma(&mut self) -> bool {
        self.eat(Token::Comma).is_some()
    }
}
