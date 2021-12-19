use dada_ir::format_string::{
    FormatString, FormatStringData, FormatStringSection, FormatStringSectionData,
};
use dada_ir::span::{Offset, Span};
use dada_ir::token::Token;
use dada_ir::token_tree::TokenTree;
use dada_ir::word::Word;
use std::iter::Peekable;

pub fn lex_file(db: &dyn crate::Db, filename: Word) -> TokenTree {
    let source_text = dada_manifest::source_text(db, filename);
    let chars = &mut source_text.char_indices().peekable();
    let mut lexer = Lexer {
        db,
        filename,
        chars,
        file_len: source_text.len(),
    };
    lexer.lex_tokens(None)
}

#[track_caller]
pub fn closing_delimiter(ch: char) -> char {
    match ch {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        _ => panic!("not a delimiter: {:?}", ch),
    }
}

macro_rules! op {
    () => {
        '+' | '-' | '/' | '*' | '>' | '<' | '&' | '|' | '.' | ',' | ':' | ';'
    };
}

struct Lexer<'me, I>
where
    I: Iterator<Item = (usize, char)>,
{
    db: &'me dyn crate::Db,
    filename: Word,
    chars: &'me mut Peekable<I>,
    file_len: usize,
}

impl<'me, I> Lexer<'me, I>
where
    I: Iterator<Item = (usize, char)>,
{
    fn lex_tokens(&mut self, end_ch: Option<char>) -> TokenTree {
        let mut tokens = vec![];
        let mut start_pos = self.file_len;
        let mut end_pos = 0;
        while let Some((pos, ch)) = self.chars.peek().cloned() {
            start_pos = start_pos.min(pos);
            end_pos = end_pos.max(pos);

            if Some(ch) == end_ch {
                break;
            }

            self.chars.next();

            match ch {
                '(' | '[' | '{' => {
                    tokens.push(Token::Delimiter(ch));
                    let tree = self.lex_tokens(Some(closing_delimiter(ch)));
                    tokens.push(Token::Tree(tree));
                }
                ')' | ']' | '}' => {
                    tokens.push(Token::Delimiter(ch));
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let text = self
                        .accumulate(ch, |c| matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9'));

                    // Check what comes next to decide if this is
                    // a "prefix" like `r'foo"` or an identifier `r`.
                    let is_prefix = self
                        .chars
                        .peek()
                        .map(|&(_, ch)| matches!(ch, '"' | '\''))
                        .unwrap_or(false);

                    if is_prefix {
                        tokens.push(Token::Prefix(text));
                    } else {
                        tokens.push(Token::Alphabetic(text));
                    }
                }
                '0'..='9' => {
                    let text = self.accumulate(ch, |c| matches!(c, '0'..='9' | '_'));
                    tokens.push(Token::Number(text));
                }
                op!() => {
                    tokens.push(Token::Op(ch));
                }
                '"' => {
                    tokens.push(Token::FormatString(self.string_literal(Offset::from(pos))));
                }
                _ => {
                    if !ch.is_whitespace() {
                        tokens.push(Token::Unknown(ch));
                    } else {
                        tokens.push(Token::Whitespace(ch));
                    }
                }
            }
        }

        if self.chars.peek().is_none() {
            end_pos = end_pos.max(self.file_len);
        }

        end_pos = end_pos.max(start_pos);

        TokenTree::new(
            self.db,
            self.filename,
            Span::from(start_pos, end_pos),
            tokens,
        )
    }

    /// Returns the offset of the next character within the file.
    fn peek_offset(&mut self) -> usize {
        match self.chars.peek() {
            Some((o, _)) => *o,
            None => self.file_len,
        }
    }

    /// Accumulate `ch0` and following characters while `matches` returns true
    /// into an interned string.
    fn accumulate(&mut self, ch0: char, matches: impl Fn(char) -> bool) -> Word {
        let mut string = String::new();
        string.push(ch0);
        while let Some(&(_, ch1)) = self.chars.peek() {
            if !matches(ch1) {
                break;
            }

            string.push(ch1);
            self.chars.next();
        }
        Word::from(self.db, string)
    }

    /// Invoked after consuming a `"`
    fn string_literal(&mut self, start: Offset) -> FormatString {
        let mut buffer = StringFormatBuffer::new(self.db);
        while let Some((ch_offset, ch)) = self.chars.next() {
            let ch_offset = Offset::from(ch_offset);

            if ch == '"' {
                break;
            }

            if ch == '{' {
                // Format string! Grab a token tree.
                let tree = self.lex_tokens(Some('}'));
                buffer.push_tree(tree);

                if let Some(&(_, '}')) = self.chars.peek() {
                    self.chars.next();
                } else {
                    let end = Offset::from(
                        self.chars
                            .peek()
                            .map(|pair| pair.0)
                            .unwrap_or(self.file_len),
                    );
                    dada_ir::error!(
                        Span {
                            start: Offset::from(ch_offset),
                            end,
                        }
                        .in_file(self.filename),
                        "format string missing closing brace in code section"
                    )
                    .emit(self.db);
                    break;
                }
                continue;
            }

            buffer.push_char(ch);
        }

        buffer.flush_text();

        let end = Offset::from(self.peek_offset());

        FormatStringData {
            len: end - start,
            sections: buffer.sections,
        }
        .intern(self.db)
    }
}

struct StringFormatBuffer<'me> {
    db: &'me dyn crate::Db,
    sections: Vec<FormatStringSection>,
    text: String,
}

impl<'me> StringFormatBuffer<'me> {
    pub fn new(db: &'me dyn crate::Db) -> Self {
        Self {
            db,
            sections: Default::default(),
            text: Default::default(),
        }
    }

    fn push_char(&mut self, ch: char) {
        self.text.push(ch);
    }

    fn push_tree(&mut self, token_tree: TokenTree) {
        self.flush_text();
        self.sections
            .push(FormatStringSectionData::TokenTree(token_tree).intern(self.db));
    }

    fn flush_text(&mut self) {
        let text = std::mem::replace(&mut self.text, String::new());
        if !text.is_empty() {
            let word = Word::from(self.db, text);
            let section = FormatStringSectionData::Text(word).intern(self.db);
            self.sections.push(section);
        }
    }
}
