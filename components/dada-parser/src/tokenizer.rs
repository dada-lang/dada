use dada_util::Map;

use dada_ir_ast::{
    ast::{Identifier, LiteralKind},
    diagnostic::Diagnostic,
    span::{Anchor, Offset, Span},
};

#[derive(Clone)]
pub struct Token<'input, 'db> {
    pub span: Span<'db>,
    pub skipped: Option<Skipped>,
    pub kind: TokenKind<'input, 'db>,
}

/// Records tokens that were skipped before this token was issued.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Skipped {
    /// Misc non-newline whitespace was skipped
    Whitespace,

    /// Whitespace including at least one `\n` was skipped
    Newline,

    /// A comment was skipped (which implies a newline)
    Comment,
}

#[derive(Clone, Debug)]
pub enum TokenKind<'input, 'db> {
    /// A program identifier
    Identifier(Identifier<'db>),

    /// A keyword
    Keyword(Keyword),

    /// A delimeted tree like `{}` or `[]` and the text that was in it.
    Delimited {
        delimiter: Delimiter,
        text: &'input str,
    },

    /// An op-char like `+`, `-`, etc.
    OpChar(char),

    /// An integer like `22`
    Literal(LiteralKind, &'input str),

    /// Invalid characters
    Error(Diagnostic),
}

macro_rules! keywords {
    (pub enum $Keyword:ident {
        $($kw:ident,)*
    }) => {
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        pub enum $Keyword {
            $($kw,)*
        }

        impl std::fmt::Display for $Keyword {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let s = match self {
                    $(Self::$kw => stringify!($kw),)*
                };
                write!(f, "`{}`", s.to_lowercase())
            }
        }

        impl $Keyword {
            const UPPER_STRINGS: &'static [(&'static str, $Keyword)] = &[
                $((stringify!($kw), $Keyword::$kw),)*
            ];
        }
    }
}

keywords! {
    pub enum Keyword {
        As,
        Class,
        Crate,
        Enum,
        Export,
        Fn,
        Lease,
        Leased,
        Let,
        Give,
        Given,
        Mod,
        My,
        Our,
        Perm,
        Pub,
        Return,
        Share,
        Shared,
        Struct,
        Type,
        Use,
        Where,
    }
}

impl Keyword {
    fn map() -> &'static Map<String, Keyword> {
        static MAP: std::sync::OnceLock<Map<String, Keyword>> = std::sync::OnceLock::new();
        MAP.get_or_init(|| {
            let mut map = Map::default();
            for (upper_str, kw) in Keyword::UPPER_STRINGS {
                map.insert(upper_str.to_lowercase(), *kw);
            }
            map
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Delimiter {
    Parentheses,
    SquareBrackets,
    CurlyBraces,
}

impl Delimiter {
    pub fn open_char(self) -> char {
        match self {
            Self::Parentheses => '(',
            Self::SquareBrackets => '[',
            Self::CurlyBraces => '{',
        }
    }

    pub fn close_char(self) -> char {
        match self {
            Self::Parentheses => ')',
            Self::SquareBrackets => ']',
            Self::CurlyBraces => '}',
        }
    }

    pub fn chars(self) -> &'static str {
        match self {
            Delimiter::Parentheses => "()",
            Delimiter::SquareBrackets => "[]",
            Delimiter::CurlyBraces => "{}",
        }
    }
}

pub fn tokenize<'input, 'db>(
    db: &'db dyn crate::Db,
    anchor: Anchor<'db>,
    input_offset: Offset,
    input: &'input str,
) -> Vec<Token<'input, 'db>> {
    Tokenizer {
        db,
        anchor,
        input,
        chars: input.char_indices().peekable(),
        tokens: vec![],
        kws: Keyword::map(),
        error_start: None,
        input_offset,
        skipped_accum: None,
    }
    .tokenize()
}

struct Tokenizer<'input, 'db> {
    db: &'db dyn crate::Db,
    anchor: Anchor<'db>,
    input: &'input str,
    chars: CharIndices<'input>,
    tokens: Vec<Token<'input, 'db>>,
    kws: &'static Map<String, Keyword>,
    input_offset: Offset,
    error_start: Option<usize>,
    skipped_accum: Option<Skipped>,
}

impl<'input, 'db> Tokenizer<'input, 'db> {
    fn tokenize(mut self) -> Vec<Token<'input, 'db>> {
        while let Some((index, ch)) = self.chars.next() {
            match ch {
                // Comments
                '#' => self.comment(index),

                // Identifiers and keywords
                _ if ch.is_alphabetic() || ch == '_' => self.identifier(index, ch),

                // Delimited
                '{' => self.delimited(index, Delimiter::CurlyBraces, '}'),
                '[' => self.delimited(index, Delimiter::SquareBrackets, ']'),
                '(' => self.delimited(index, Delimiter::Parentheses, ')'),

                // Integers
                _ if ch.is_digit(10) => self.integer(index, ch),

                // Whitespace
                _ if ch.is_whitespace() => {
                    self.accumulate_skipped(Skipped::Whitespace);
                }

                // Ops
                _ if is_op_char(ch) => self.ops(index, ch),

                _ => {
                    // Record start of an errorneous set of tokens.
                    // When we reach the start of a valid token (or end of input)
                    // this will be reported as an error in `clear_accumulated`.
                    if self.error_start.is_none() {
                        self.error_start = Some(index);
                    }
                }
            }
        }

        let _skipped = self.clear_accumulated(self.input.len());

        self.tokens
    }

    fn accumulate_skipped(&mut self, skipped: Skipped) {
        self.skipped_accum = std::cmp::max(self.skipped_accum, Some(skipped));
    }

    /// Clears various accumulated state in prep for a new token being issued (or the final token).
    /// Returns the [`Skipped`][] value that should be used for the next token issued (if any).
    /// Reports errors for any invalid characters seen thus far.
    fn clear_accumulated(&mut self, index: usize) -> Option<Skipped> {
        if let Some(start) = self.error_start {
            self.error_start = None;

            let span = self.span(start, index);
            self.tokens.push(Token {
                span,
                skipped: None,
                kind: TokenKind::Error(Diagnostic::error(self.db, span, "invalid token(s)")),
            });
        }

        self.skipped_accum.take()
    }

    fn span(&self, start: usize, end: usize) -> Span<'db> {
        assert!(end >= start);
        Span {
            anchor: self.anchor,
            start: self.input_offset + start,
            end: self.input_offset + end,
        }
    }

    fn comment(&mut self, index: usize) {
        let _skipped = self.clear_accumulated(index);
        self.accumulate_skipped(Skipped::Comment);

        for (_index, ch) in &mut self.chars {
            if ch == '\n' {
                return;
            }
        }
    }

    fn identifier(&mut self, start: usize, ch: char) {
        let skipped = self.clear_accumulated(start);

        let mut end = start + ch.len_utf8();

        while let Some(&(index, ch)) = self.chars.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                end = index + ch.len_utf8();
                self.chars.next();
            } else {
                break;
            }
        }

        let span = Span {
            anchor: self.anchor,
            start: Offset::from(start),
            end: Offset::from(end),
        };

        let text = &self.input[start..end];
        if let Some(kw) = self.kws.get(text) {
            self.tokens.push(Token {
                span,
                skipped,
                kind: TokenKind::Keyword(*kw),
            });
        } else {
            let identifier = Identifier::new(self.db, text.to_string());
            self.tokens.push(Token {
                span,
                skipped,
                kind: TokenKind::Identifier(identifier),
            })
        }
    }

    fn integer(&mut self, start: usize, ch: char) {
        let skipped = self.clear_accumulated(start);

        let mut end = start + ch.len_utf8();

        while let Some(&(index, ch)) = self.chars.peek() {
            if ch.is_digit(10) || ch == '_' {
                end = index + ch.len_utf8();
                self.chars.next();
            } else {
                break;
            }
        }

        let span = Span {
            anchor: self.anchor,
            start: Offset::from(start),
            end: Offset::from(end),
        };

        let text = &self.input[start..end];
        self.tokens.push(Token {
            span,
            skipped,
            kind: TokenKind::Literal(LiteralKind::Integer, text),
        });
    }

    fn delimited(&mut self, start: usize, delim: Delimiter, close: char) {
        let skipped = self.clear_accumulated(start);

        while let Some((end, ch)) = self.chars.next() {
            match ch {
                _ if ch == close => {
                    assert!(ch.len_utf8() == 1);
                    self.tokens.push(Token {
                        span: self.span(start, end),
                        skipped,
                        kind: TokenKind::Delimited {
                            delimiter: delim,
                            text: &self.input[start + 1..end],
                        },
                    });
                    break;
                }
                _ => {}
            }
        }
    }

    fn ops(&mut self, start: usize, ch: char) {
        let skipped = self.clear_accumulated(start);
        self.chars.next();
        self.tokens.push(Token {
            span: self.span(start, start + ch.len_utf8()),
            skipped,
            kind: TokenKind::OpChar(ch),
        });
    }
}

fn is_op_char(ch: char) -> bool {
    matches!(
        ch,
        '+' | '-' | '*' | '/' | '%' | '=' | '!' | '<' | '>' | '&' | '|',
    )
}

type CharIndices<'input> = std::iter::Peekable<std::str::CharIndices<'input>>;
