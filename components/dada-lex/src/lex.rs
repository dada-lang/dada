use dada_ir::span::Span;
use dada_ir::token::Token;
use dada_ir::token_tree::TokenTree;
use dada_ir::word::Word;
use std::iter::Peekable;

pub fn lex_file(db: &dyn crate::Db, filename: Word) -> TokenTree {
    let source_text = dada_manifest::source_text(db, filename);
    let chars = &mut source_text.char_indices().peekable();
    lex_tokens(db, filename, chars, source_text.len(), None)
}

macro_rules! op {
    () => {
        '+' | '-' | '/' | '*' | '>' | '<' | '&' | '|' | '.' | ',' | ':' | ';'
    };
}

fn lex_tokens(
    db: &dyn crate::Db,
    filename: Word,
    chars: &mut Peekable<impl Iterator<Item = (usize, char)>>,
    file_len: usize,
    end_ch: Option<char>,
) -> TokenTree {
    let mut tokens = vec![];
    let mut start_pos = file_len;
    let mut end_pos = file_len;
    while let Some((pos, ch)) = chars.peek().cloned() {
        start_pos = start_pos.min(pos);
        end_pos = end_pos.max(pos);

        if Some(ch) == end_ch {
            break;
        }

        chars.next();

        match ch {
            '(' | '[' | '{' => {
                tokens.push(Token::Delimiter(ch));
                let tree = lex_tokens(db, filename, chars, file_len, Some(closing_delimiter(ch)));
                tokens.push(Token::Tree(tree));
            }
            ')' | ']' | '}' => {
                tokens.push(Token::Delimiter(ch));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let text = accumulate(
                    db,
                    ch,
                    chars,
                    |c| matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9'),
                );
                tokens.push(Token::Alphabetic(text));
            }
            '0'..='9' => {
                let text = accumulate(db, ch, chars, |c| matches!(c, '0'..='9' | '_'));
                tokens.push(Token::Number(text));
            }
            op!() => {
                tokens.push(Token::Op(ch));
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

    TokenTree::new(db, filename, Span::from(start_pos, end_pos), tokens)
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

fn accumulate(
    db: &dyn crate::Db,
    ch0: char,
    chars: &mut Peekable<impl Iterator<Item = (usize, char)>>,
    matches: impl Fn(char) -> bool,
) -> Word {
    let mut string = String::new();
    string.push(ch0);
    while let Some(&(_, ch1)) = chars.peek() {
        if !matches(ch1) {
            break;
        }

        string.push(ch1);
        chars.next();
    }
    Word::from(db, string)
}
