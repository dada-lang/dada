#![feature(trait_upcasting)]
#![allow(incomplete_features)]

use dada_ir::span::Span;
use dada_ir::token::Token;
use dada_ir::token_tree::TokenTree;
use dada_ir::word::Word;
use std::iter::Peekable;

#[salsa::jar(Db)]
pub struct Jar(lex_file);

pub trait Db: salsa::DbWithJar<Jar> + dada_manifest::Db + dada_ir::Db {
    fn lex(&self) -> &dyn Db;
}
impl<T> Db for T
where
    T: salsa::DbWithJar<Jar> + dada_manifest::Db + dada_ir::Db,
{
    fn lex(&self) -> &dyn Db {
        self
    }
}

#[salsa::memoized(in Jar)]
pub fn lex_file(db: &dyn Db, filename: Word) -> TokenTree {
    let source_text = dada_manifest::source_text(db, filename);
    let chars = &mut source_text.char_indices().peekable();
    lex_tokens(db, chars, source_text.len(), None)
}

macro_rules! op {
    () => {
        '+' | '-' | '/' | '*' | '>' | '<' | '&' | '|' | '.' | ',' | ':' | ';'
    };
}

fn lex_tokens(
    db: &dyn Db,
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
                let tree = lex_tokens(db, chars, file_len, Some(closing_delimiter(ch)));
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
                if let Some(&(_, op!())) = chars.peek() {
                    // Followed by another operator
                    tokens.push(Token::OpAdjacent(ch));
                } else {
                    // Not followed by another operator
                    tokens.push(Token::OpAlone(ch));
                }
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

    TokenTree::new(db, tokens, Span::from(start_pos, end_pos))
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
    db: &dyn Db,
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
