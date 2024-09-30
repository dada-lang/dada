use salsa::Update;
use tokenizer::{tokenize, Delimiter, Keyword, Skipped, Token, TokenKind};

use dada_ir_ast::{
    ast::{AstModule, SpanVec, SpannedIdentifier},
    diagnostic::{Diagnostic, Reported},
    inputs::SourceFile,
    span::{Anchor, Offset, Span},
};

use salsa::Database as Db;

mod class_body;
mod expr;
mod function_body;
mod generics;
mod miscellaneous;
mod module_body;
pub mod prelude;
mod tokenizer;
mod types;

#[salsa::tracked]
impl prelude::SourceFileParse for SourceFile {
    #[salsa::tracked]
    fn parse(self, db: &dyn crate::Db) -> AstModule<'_> {
        let anchor = Anchor::SourceFile(self);
        let text = self.contents(db);
        let tokens = tokenizer::tokenize(db, anchor, Offset::ZERO, text);
        let mut parser = Parser::new(db, anchor, &tokens);
        let module = AstModule::eat(db, &mut parser).expect("parsing a module is infallible");
        parser.into_diagnostics(db).into_iter().for_each(|d| {
            let Reported = d.report(db);
        });
        module
    }
}

struct Parser<'token, 'db> {
    /// Input tokens
    tokens: &'token [Token<'token, 'db>],

    /// Next token (if any) in the token list
    next_token: usize,

    /// Span of the last consumed token; starts as the span of the anchor
    last_span: Span<'db>,

    /// Additional diagnostics that were reported by parsers.
    /// Used when we are able to partially parse something and recover.
    /// These need to be reported to the user eventually.
    /// They are stored in the parser to support speculative parsing.
    diagnostics: Vec<Diagnostic>,
}

impl<'token, 'db> Parser<'token, 'db> {
    pub fn new(
        _db: &'db dyn crate::Db,
        anchor: Anchor<'db>,
        tokens: &'token [Token<'token, 'db>],
    ) -> Self {
        let mut this = Self {
            tokens,
            next_token: 0,
            last_span: Span {
                anchor,
                start: Offset::ZERO,
                end: Offset::ZERO,
            },
            diagnostics: Vec::new(),
        };

        this.eat_errors();

        this
    }

    /// Top-level parsing function: parses zero or more instances of T and reports any errors.
    pub fn parse_many_and_report_diagnostics<T>(
        mut self,
        db: &'db dyn crate::Db,
    ) -> SpanVec<'db, T::Output>
    where
        T: Parse<'db>,
    {
        let start_span = self.peek_span();

        let result = match T::eat_many(db, &mut self) {
            Ok(v) => v,
            Err(err) => {
                self.push_diagnostic(err.into_diagnostic(db));
                SpanVec {
                    span: start_span.to(self.last_span()),
                    values: vec![],
                }
            }
        };

        for diagnostic in self.into_diagnostics(db) {
            diagnostic.report(db);
        }

        result
    }

    /// Record a diagnostic, indicating that parsing recovered from an error.
    pub fn push_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Take all diagnostics from another parser (e.g., one parsing a delimited set of tokens).
    pub fn take_diagnostics(&mut self, db: &'db dyn crate::Db, parser: Parser<'_, 'db>) {
        self.diagnostics.extend(parser.into_diagnostics(db));
    }

    /// Complete parsing and convert the parser into the resulting diagnostics (errors).
    ///
    /// Reports an error if there are any unconsumed tokens.
    pub fn into_diagnostics(mut self, db: &'db dyn crate::Db) -> Vec<Diagnostic> {
        if self.peek().is_some() {
            let diagnostic = self.illformed(Expected::EOF).into_diagnostic(db);
            self.push_diagnostic(diagnostic);

            // consume all remaining tokens lest there is a tokenizer error in there
            while self.peek().is_some() {
                self.eat_next_token().unwrap();
            }
        }

        self.diagnostics
    }

    /// Forks this parser into a split parser at the same point
    /// with a fresh set of diagnostics. Used for speculation.
    fn fork(&self) -> Self {
        Self {
            tokens: self.tokens,
            next_token: self.next_token,
            last_span: self.last_span,
            diagnostics: Vec::new(),
        }
    }

    /// Eat any pending errors and add them to the list of errors to report.
    /// Does not adjust `last_span`.
    ///
    /// Invoked automatically after each call to `eat_next_token`.
    fn eat_errors(&mut self) {
        while let Some(Token {
            kind: TokenKind::Error(diagnostic),
            ..
        }) = self.tokens.get(self.next_token)
        {
            self.push_diagnostic(diagnostic.clone());
            self.next_token += 1;
        }
    }

    /// Advance by one token, returning `Err` if there is no current token.
    /// After advancing, also eagerly eats any error tokens.
    pub fn eat_next_token(&mut self) -> Result<(), ParseFail<'db>> {
        if self.next_token < self.tokens.len() {
            assert!(self.next_token < self.tokens.len());
            let span = self.tokens[self.next_token].span;
            assert_eq!(span.anchor, self.last_span.anchor);
            self.last_span = span;
            self.next_token += 1;
            self.eat_errors();
            Ok(())
        } else {
            Err(self.illformed(Expected::MoreTokens))
        }
    }

    /// Peek at the next token, returning None if there is none.
    /// Implicitly advances past error tokens.
    /// Does not consume the token returned.
    pub fn peek(&mut self) -> Option<&Token<'token, 'db>> {
        let token = self.tokens.get(self.next_token)?;

        assert!(!matches!(
            token,
            Token {
                kind: TokenKind::Error(_),
                ..
            },
        ));

        Some(token)
    }

    /// Span of the last consumed token.
    pub fn last_span(&self) -> Span<'db> {
        self.last_span
    }

    /// Span of the next token in the input (or the end of the final token, if there are no more tokens)
    pub fn peek_span(&mut self) -> Span<'db> {
        let s = match self.peek() {
            Some(token) => token.span,
            None => self.last_span.at_end(),
        };
        assert_eq!(s.anchor, self.last_span.anchor);
        s
    }

    pub fn illformed(&mut self, expected: Expected) -> ParseFail<'db> {
        ParseFail {
            span: self.peek_span(),
            expected,
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
                self.eat_next_token().unwrap();
                return Ok(span);
            }
        }
        Err(self.illformed(Expected::Keyword(kw)))
    }

    pub fn eat_id(&mut self) -> Result<SpannedIdentifier<'db>, ParseFail<'db>> {
        if let Some(&Token {
            kind: TokenKind::Identifier(id),
            span,
            skipped: _,
        }) = self.peek()
        {
            self.eat_next_token().unwrap();
            return Ok(SpannedIdentifier { span, id });
        }
        Err(self.illformed(Expected::Identifier))
    }

    pub fn eat_op(&mut self, chars: &'static str) -> Result<Span<'db>, ParseFail<'db>> {
        let mut iter = chars.chars();

        let ch = iter.next().unwrap();

        // First character can have any skipped
        let Some(&Token {
            kind: TokenKind::OpChar(ch1),
            span: start_span,
            skipped: _,
        }) = self.peek()
        else {
            return Err(self.illformed(Expected::Operator(chars)));
        };

        if ch != ch1 {
            return Err(self.illformed(Expected::Operator(chars)));
        }

        self.eat_next_token().unwrap();

        for ch in iter {
            let Some(&Token {
                kind: TokenKind::OpChar(ch1),
                skipped,
                span: _,
            }) = self.peek()
            else {
                return Err(self.illformed(Expected::Operator(chars)));
            };

            if ch != ch1 || skipped >= Some(Skipped::Newline) {
                return Err(self.illformed(Expected::Operator(chars)));
            }

            self.eat_next_token().unwrap();
        }

        Ok(start_span.to(self.last_span()))
    }

    pub fn eat_delimited(&mut self, delimiter: Delimiter) -> Result<&'token str, ParseFail<'db>> {
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
                self.eat_next_token().unwrap();
                return Ok(text);
            }
        }

        Err(self.illformed(Expected::Delimited(delimiter)))
    }

    /// Returns true if the next token is on the same line
    /// as the most recently consumed token.
    /// Some parts of our grammar are newline sensitive.
    fn next_token_on_same_line(&mut self) -> bool {
        match self.peek() {
            Some(Token { skipped, .. }) => match skipped {
                Some(skipped) => *skipped <= Skipped::Whitespace,
                None => true,
            },
            None => false,
        }
    }
}

/// Parse an instance of `Self` from the given [`Parser`][].
///
/// There are several parsing methods depending on how many instances of `Self` you wish to parse:
///
/// * [`opt_parse`](Parse::opt_parse) -- 0 or 1 instance (`x?` in a regex)
/// * [`opt_parse_comma`](Parse::opt_parse) -- comma-separated list, returns `None` if no elements found
/// * [`opt_parse_delimited`](Parse::opt_parse_delimited) -- delimited comma-separated list, `None` if no delimiters are found
/// * [`eat`](Parse::eat) -- exactly 1 instance (`x` in a regex`)
/// * [`eat_comma`](Parse::eat_comma) -- comma-separated list, returns an empty list if no elements found
/// * [`eat_delimited`](Parse::eat_delimited) -- delimited comma-separated list where delimiters are mandatory
///
/// Implementors need only provide `opt_parse`, the rest are automatically provided.
///
/// # Return values
///
/// The `opt_parse` methods return an `Result<Option<_>, ParseFail<'db>>` as follows:
///
/// * `Ok(Some(v))` -- parse succeeded (possibly with recovery,
///   in which case diagnostics will be stored into the [`Parser`][]).
/// * `Ok(None)` -- no instance of `Self` was found.
/// * `Err(err)` -- a malformed instance of `Self` was found. Some tokens were consumed.
///
/// The `eat` methods return a `Result<_, ParseFail<'db>>` and hence only distinguish success and error.
///
/// # Diagnostics
///
/// Parsing something **can never** report diagnostics to the user.
/// Any diagnostics need to be accumulated in the [`Parser`][].
#[allow(dead_code)] // some fns not currently used
trait Parse<'db>: Sized {
    type Output: Update;

    /// Speculatively parses to see if we could eat a `Self`
    /// without any error. Never consumes tokens nor produces an error.
    fn can_eat(db: &'db dyn crate::Db, parser: &Parser<'_, 'db>) -> bool {
        let mut parser = parser.fork();
        match Self::eat(db, &mut parser) {
            Ok(_) => parser.diagnostics.is_empty(),
            Err(_) => false,
        }
    }

    /// Parses an instance of `Self` from the given [`Parser`][], reporting an error if no instance is found.
    fn eat(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Self::Output, ParseFail<'db>> {
        match Self::opt_parse(db, parser)? {
            Some(v) => Ok(v),
            None => Err(parser.illformed(Self::expected())),
        }
    }

    /// Parse zero-or-more comma-separated instances of `Self` returning a vector.
    /// Accepts trailing commas.
    fn eat_comma(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<SpanVec<'db, Self::Output>, ParseFail<'db>> {
        match Self::opt_parse_comma(db, parser)? {
            Some(v) => Ok(v),
            None => Ok(SpanVec {
                span: parser.last_span().at_end(),
                values: vec![],
            }),
        }
    }

    /// Parse zero-or-more instances of `Self` returning a vector.
    fn eat_many(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<SpanVec<'db, Self::Output>, ParseFail<'db>> {
        let mut values = vec![];
        let start_span = parser.peek_span();
        loop {
            match Self::opt_parse(db, parser) {
                Ok(Some(v)) => values.push(v),
                Ok(None) => break,
                Err(err) if values.is_empty() => return Err(err),
                Err(err) => {
                    parser.push_diagnostic(err.into_diagnostic(db));
                    break;
                }
            }
        }

        Ok(SpanVec {
            span: start_span.to(parser.last_span()),
            values,
        })
    }

    /// Eat a comma separated list of Self, delimited by `delimiter`
    /// (e.g., `(a, b, c)`).
    fn eat_delimited<T>(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
        delimiter: Delimiter,
        eat_method: impl FnOnce(&'db dyn crate::Db, &mut Parser<'_, 'db>) -> Result<T, ParseFail<'db>>,
    ) -> Result<T, ParseFail<'db>> {
        match Self::opt_parse_delimited(db, parser, delimiter, eat_method)? {
            Some(v) => Ok(v),
            None => Err(parser.illformed(Expected::Delimited(delimiter))),
        }
    }

    /// Parse a single instance of `Self`, returning `Ok(Some(v))`.
    /// Returns `Ok(None)` if `Self` was not present or `Err(err)`
    /// if `Self` appeared to be present but was ill-formed.
    ///
    /// Invariants maintained by this method:
    ///
    /// * If `Ok(None)` is returned, consumed *NO* tokens and reported *NO* diagnostics.
    /// * If `Err` is returned, consumed at least one token (not true for `eat` methods).
    fn opt_parse(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self::Output>, ParseFail<'db>>;

    /// Parse a delimited list of Self
    /// e.g., `(a, b, c)` or `[a, b, c]`. Returns `None` if
    /// the given delimiters indicated by `delimiter` are not found.
    fn opt_parse_delimited<T>(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
        delimiter: Delimiter,
        eat_method: impl FnOnce(&'db dyn crate::Db, &mut Parser<'_, 'db>) -> Result<T, ParseFail<'db>>,
    ) -> Result<Option<T>, ParseFail<'db>> {
        let Ok(text) = parser.eat_delimited(delimiter) else {
            return Ok(None);
        };

        let text_span = parser.last_span();
        let input_offset = text_span.start + 1; // account for the opening delimiter
        let tokenized = tokenize(db, text_span.anchor, input_offset, text);
        let mut parser1 = Parser::new(db, text_span.anchor, &tokenized);
        let opt_list_err = eat_method(db, &mut parser1);
        parser.take_diagnostics(db, parser1);
        Ok(Some(opt_list_err?))
    }

    /// Parse a comma separated list of Self
    fn opt_parse_comma(
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<SpanVec<'db, Self::Output>>, ParseFail<'db>> {
        match Self::opt_parse(db, parser) {
            Ok(Some(item)) => {
                let mut values = vec![item];

                while parser.eat_op(",").is_ok() {
                    match Self::opt_parse(db, parser) {
                        Ok(Some(item)) => values.push(item),
                        Ok(None) => break,
                        Err(err) => {
                            parser.push_diagnostic(err.into_diagnostic(db));
                            break;
                        }
                    }
                }

                Ok(Some(SpanVec {
                    span: parser.last_span(),
                    values,
                }))
            }

            Ok(None) => Ok(None),

            Err(err) => Err(err),
        }
    }

    /// If `guard_op` appears, then parse `Self`
    fn opt_parse_guarded(
        guard_op: impl ParseGuard,
        db: &'db dyn crate::Db,
        parser: &mut Parser<'_, 'db>,
    ) -> Result<Option<Self::Output>, ParseFail<'db>> {
        if guard_op.eat(db, parser) {
            Ok(Some(Self::eat(db, parser)?))
        } else {
            Ok(None)
        }
    }

    fn expected() -> Expected;
}

trait ParseGuard {
    fn eat<'db>(self, db: &'db dyn crate::Db, parser: &mut Parser<'_, 'db>) -> bool;
}

impl ParseGuard for &'static str {
    fn eat<'db>(self, _db: &'db dyn crate::Db, parser: &mut Parser<'_, 'db>) -> bool {
        parser.eat_op(self).is_ok()
    }
}

impl ParseGuard for Keyword {
    fn eat<'db>(self, _db: &'db dyn crate::Db, parser: &mut Parser<'_, 'db>) -> bool {
        parser.eat_keyword(self).is_ok()
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct ParseFail<'db> {
    span: Span<'db>,
    expected: Expected,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Expected {
    EOF,
    MoreTokens,
    Identifier,
    Operator(&'static str),
    Keyword(Keyword),
    Delimited(Delimiter),
    Nonterminal(&'static str),
}

impl<'db> ParseFail<'db> {
    pub fn into_diagnostic(self, db: &dyn crate::Db) -> Diagnostic {
        let message = match self.expected {
            Expected::EOF => "expected end of input".to_string(),
            Expected::MoreTokens => "expected more tokens".to_string(),
            Expected::Identifier => "expected an identifier".to_string(),
            Expected::Operator(op) => format!("expected `{op}`"),
            Expected::Keyword(k) => format!("expected `{k:?}`"),
            Expected::Delimited(d) => format!("expected `{}`", d.open_char()),
            Expected::Nonterminal(n) => format!("expected {n}"),
        };
        Diagnostic::error(db, self.span, message)
    }
}
