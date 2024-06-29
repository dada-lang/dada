use dada_util::Map;
use salsa::DebugWithDb;

use crate::{
    ast::{Identifier, Item},
    diagnostic,
    span::{Offset, Span},
};

#[derive(Clone, Copy)]
pub struct Token<'input, 'db> {
    pub span: Span<'db>,
    pub kind: TokenKind<'input, 'db>,
}

#[derive(Clone, Copy)]
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
    OpChar {
        /// If true, then this operator was adjacent to an operator on the left (the previous token).
        adjacent_left: bool,

        /// The "operator" character (e.g., `+`).
        ch: char,

        /// If true, then this operator was adjacent to an operator on the right (the next token).
        adjacent_right: bool,
    },
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
        Fn,
        Class,
        Struct,
        Enum,
        Share,
        Shared,
        Lease,
        Leased,
        Give,
        Given,
        My,
        Our,
        Where,
        Use,
        As,
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

pub fn tokenize<'input, 'db>(
    db: &'db dyn crate::Db,
    anchor: Item<'db>,
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
    }
    .tokenize()
}

struct Tokenizer<'input, 'db> {
    db: &'db dyn crate::Db,
    anchor: Item<'db>,
    input: &'input str,
    chars: CharIndices<'input>,
    tokens: Vec<Token<'input, 'db>>,
    kws: &'static Map<String, Keyword>,
    input_offset: Offset,
    error_start: Option<usize>,
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

                // Whitespace
                _ if ch.is_whitespace() => {}

                // Ops
                _ if is_op_char(ch) => self.ops(index, ch),

                _ => {
                    if self.error_start.is_none() {
                        self.error_start = Some(index);
                    }
                }
            }
        }

        self.clear_error(self.input.len());

        self.tokens
    }

    fn clear_error(&mut self, index: usize) {
        let Some(start) = self.error_start else {
            return;
        };

        self.error_start = None;

        let span = self.span(start, index);
        diagnostic::report_error(self.db, span, format!("invalid token(s)"));
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
        self.clear_error(index);

        while let Some((_index, ch)) = self.chars.next() {
            if ch == '\n' {
                return;
            }
        }
    }

    fn identifier(&mut self, start: usize, ch: char) {
        self.clear_error(start);

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
                kind: TokenKind::Keyword(*kw),
            });
        } else {
            let identifier = Identifier::new(self.db, text.to_string());
            self.tokens.push(Token {
                span,
                kind: TokenKind::Identifier(identifier),
            })
        }
    }

    fn delimited(&mut self, start: usize, delim: Delimiter, close: char) {
        self.clear_error(start);

        while let Some((end, ch)) = self.chars.next() {
            match ch {
                _ if ch == close => {
                    assert!(ch.len_utf8() == 1);
                    self.tokens.push(Token {
                        span: self.span(start, end),
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
        self.clear_error(start);

        let mut ops = Vec::with_capacity(16);
        ops.push(ch);

        while let Some(&(_index, next_ch)) = self.chars.peek() {
            if is_op_char(next_ch) {
                ops.push(ch);
                self.chars.next();
            } else {
                break;
            }
        }

        let mut p = start;
        for (op, index) in ops.iter().zip(0..) {
            self.tokens.push(Token {
                span: self.span(p, p + op.len_utf8()),
                kind: TokenKind::OpChar {
                    adjacent_left: index > 0,
                    ch: *op,
                    adjacent_right: index < ops.len() - 1,
                },
            });
            p += op.len_utf8();
        }
    }
}

fn is_op_char(ch: char) -> bool {
    match ch {
        '+' | '-' | '*' | '/' | '%' | '=' | '!' | '<' | '>' | '&' | '|' => true,
        _ => false,
    }
}

type CharIndices<'input> = std::iter::Peekable<std::str::CharIndices<'input>>;
