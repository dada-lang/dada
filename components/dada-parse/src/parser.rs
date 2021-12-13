use crate::{token_test::*, tokens::Tokens};

use dada_ir::{
    diagnostic::Diagnostic, span::Span, token::Token, token_tree::TokenTree, word::Word,
};

mod items;
pub(crate) struct Parser<'db> {
    db: &'db dyn crate::Db,
    filename: Word,
    tokens: Tokens<'db>,
    errors: Vec<Diagnostic>,
}

impl<'db> Parser<'db> {
    pub(crate) fn new(db: &'db dyn crate::Db, filename: Word, tokens: Tokens<'db>) -> Self {
        Self {
            db,
            filename,
            tokens,
            errors: vec![],
        }
    }

    /// Returns Some if the next pending token matches `is`, along
    /// with the narrowed view of the next token.
    fn peek_if<TT: TokenTest>(&mut self, is: TT) -> Option<TT::Narrow> {
        let token = self.tokens.peek()?;
        is.test(self.db, token)
    }

    /// If the next pending token matches `is`, consumes it and
    /// returns the span + narrowed view. Otherwise returns None
    /// and has no effect. Returns None if there is no pending token.
    fn eat_if<TT: TokenTest>(&mut self, is: TT) -> Option<(Span, TT::Narrow)> {
        let narrow = self.peek_if(is)?;
        self.tokens.consume();
        Some((self.tokens.last_span(), narrow))
    }

    pub(crate) fn into_errors(self) -> Vec<Diagnostic> {
        self.errors
    }

    fn delimited(&mut self, delimiter: char) -> Option<TokenTree> {
        self.eat_if(Token::Delimiter(delimiter))?;

        // Lexer always produces a token tree as the next token after a delimiter:
        let (_, token_tree) = self.eat_if(AnyTree).unwrap();

        // Consume closing delimiter (if present)
        let closing_delimiter = dada_lex::closing_delimiter(delimiter);
        self.eat_if(Token::Delimiter(closing_delimiter))
            .or_report_error(self, || format!("expected `{closing_delimiter}`"));

        Some(token_tree)
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

        let span = parser.tokens.peek_span();
        parser.errors.push(Diagnostic {
            filename: parser.filename,
            span,
            message: message(),
        });
        None
    }
}
