use dada_ir::format_string::{FormatString, FormatStringSection, FormatStringSectionData};
use dada_ir::input_file::InputFile;
use dada_ir::span::{FileSpan, Offset, Span};
use dada_ir::token::Token;
use dada_ir::token_tree::TokenTree;
use dada_ir::word::Word;
use std::iter::Peekable;

pub fn lex_file(db: &dyn crate::Db, input_file: InputFile) -> TokenTree {
    let source_text = input_file.source_text(db);
    lex_text(db, input_file, source_text, 0)
}

pub(crate) fn lex_filespan(db: &dyn crate::Db, span: FileSpan) -> TokenTree {
    let source_text = span.input_file.source_text(db);
    let start = usize::from(span.start);
    let end = usize::from(span.end);
    lex_text(db, span.input_file, &source_text[start..end], start)
}

fn lex_text(
    db: &dyn crate::Db,
    input_file: InputFile,
    source_text: &str,
    start_offset: usize,
) -> TokenTree {
    let chars = &mut source_text
        .char_indices()
        .map(|(offset, ch)| (offset + start_offset, ch))
        .inspect(|pair| tracing::debug!("lex::next = {pair:?}"))
        .peekable();
    let mut lexer = Lexer {
        db,
        input_file,
        chars,
        file_len: start_offset + source_text.len(),
    };
    lexer.lex_tokens(None)
}

#[track_caller]
pub fn closing_delimiter(ch: char) -> char {
    match ch {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        _ => panic!("not a delimiter: {ch:?}"),
    }
}

macro_rules! op {
    () => {
        '+' | '-' | '/' | '*' | '>' | '<' | '&' | '|' | '.' | ':' | ';' | '='
    };
}

struct Lexer<'me, I>
where
    I: Iterator<Item = (usize, char)>,
{
    db: &'me dyn crate::Db,
    input_file: InputFile,
    chars: &'me mut Peekable<I>,
    file_len: usize,
}

impl<'me, I> Lexer<'me, I>
where
    I: Iterator<Item = (usize, char)>,
{
    #[tracing::instrument(level = "debug", skip(self))]
    fn lex_tokens(&mut self, end_ch: Option<char>) -> TokenTree {
        let mut tokens = vec![];
        let mut push_token = |t: Token| {
            tracing::debug!("push token: {:?}", t);
            tokens.push(t);
        };
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
                    push_token(Token::Delimiter(ch));
                    let closing_ch = closing_delimiter(ch);
                    let tree = self.lex_tokens(Some(closing_ch));
                    push_token(Token::Tree(tree));

                    if let Some((_, next_ch)) = self.chars.peek() {
                        if *next_ch == closing_ch {
                            self.chars.next();
                            push_token(Token::Delimiter(closing_ch));
                        }
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let text = self
                        .accumulate(ch, |c| matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9'));

                    // Check what comes next to decide if this is
                    // a "prefix" like `r"foo"` or an identifier `r`.
                    let is_prefix = self
                        .chars
                        .peek()
                        .map(|&(_, ch)| matches!(ch, '"' | '\''))
                        .unwrap_or(false);

                    if is_prefix {
                        push_token(Token::Prefix(text));
                    } else {
                        push_token(Token::Alphabetic(text));
                    }
                }
                '#' => {
                    let s = self.accumulate_string(ch, |c| c != '\n');
                    let len: u32 = s.len().try_into().unwrap();
                    push_token(Token::Comment(len));
                }
                ',' => {
                    push_token(Token::Comma);
                }
                '0'..='9' => {
                    let text = self.accumulate(ch, |c| matches!(c, '0'..='9' | '_'));
                    push_token(Token::Number(text));
                }
                op!() => {
                    push_token(Token::Op(ch));
                }
                '"' => {
                    push_token(Token::FormatString(self.string_literal(Offset::from(pos))));
                }
                _ => {
                    if !ch.is_whitespace() {
                        push_token(Token::Unknown(ch));
                    } else {
                        push_token(Token::Whitespace(ch));
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
            self.input_file,
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
    /// into a string.
    fn accumulate_string(&mut self, ch0: char, matches: impl Fn(char) -> bool) -> String {
        let mut string = String::new();
        string.push(ch0);
        while let Some(&(_, ch1)) = self.chars.peek() {
            if !matches(ch1) {
                break;
            }

            string.push(ch1);
            self.chars.next();
        }
        string
    }

    /// Like [`Self::accumulate_string`], but interns the result.
    fn accumulate(&mut self, ch0: char, matches: impl Fn(char) -> bool) -> Word {
        let string = self.accumulate_string(ch0, matches);
        Word::intern(self.db, string)
    }

    /// Invoked after consuming a `"`
    fn string_literal(&mut self, start: Offset) -> FormatString {
        let mut buffer = StringFormatBuffer::new(self.db);
        let mut is_backslash_previous = false;
        while let Some((ch_offset, ch)) = self.chars.next() {
            let ch_offset = Offset::from(ch_offset);

            if ch == '"' && !is_backslash_previous {
                break;
            }

            if ch == '{' && !is_backslash_previous {
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
                            start: ch_offset,
                            end,
                        }
                        .anchor_to(self.db, self.input_file),
                        "format string missing closing brace in code section"
                    )
                    .emit(self.db);
                    break;
                }
                continue;
            }

            if ch == '\\' {
                is_backslash_previous = !is_backslash_previous;
            } else {
                is_backslash_previous = false;
            }

            buffer.push_char(ch);
        }

        buffer.flush_text();

        let end = Offset::from(self.peek_offset());

        FormatString::new(self.db, end - start, buffer.sections)
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
        self.sections.push(FormatStringSection::new(
            self.db,
            FormatStringSectionData::TokenTree(token_tree),
        ));
    }

    fn flush_text(&mut self) {
        let text = std::mem::take(&mut self.text);
        if !text.is_empty() {
            let word = Word::intern(self.db, text);
            let section = FormatStringSection::new(self.db, FormatStringSectionData::Text(word));
            self.sections.push(section);
        }
    }
}
