use dada_collections::Map;
use dada_ir::span::Span;
use dada_ir::word::Word;
use std::iter::Peekable;

#[salsa::jar(Lexer)]
pub struct Jar(token_tree::TokenTree, lex, kw::keywords);

pub trait Lexer: salsa::DbWithJar<Jar> + dada_manifest::Manifest + dada_ir::Ir {
    fn keywords(&self) -> &Map<Word, kw::Keyword>;
}
impl<T> Lexer for T
where
    T: salsa::DbWithJar<Jar> + dada_manifest::Manifest + dada_ir::Ir,
{
    fn keywords(&self) -> &Map<Word, kw::Keyword> {
        kw::keywords::get(self)
    }
}

pub mod kw;
pub mod token;
pub mod token_tree;
use token::Token;

#[salsa::memoized(in Jar)]
pub fn lex(db: &dyn Lexer, filename: Word) -> token_tree::TokenTree {
    let source_text = db.source_text(filename);
    let chars = &mut source_text.char_indices().peekable();
    lex_tokens(db, chars, source_text.len(), None)
}

macro_rules! op {
    () => {
        '+' | '-' | '/' | '*' | '>' | '<' | '&' | '|' | '.' | ',' | ':' | ';'
    };
}

fn lex_tokens(
    db: &dyn Lexer,
    chars: &mut Peekable<impl Iterator<Item = (usize, char)>>,
    file_len: usize,
    end_ch: Option<char>,
) -> token_tree::TokenTree {
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
                tokens.push(Token::Delimeter(ch));
                let tree = lex_tokens(db, chars, file_len, Some(closing_delimeter(ch)));
                tokens.push(Token::Tree(tree));
            }
            ')' | ']' | '}' => {
                tokens.push(Token::Delimeter(ch));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let text = accumulate(
                    db,
                    ch,
                    chars,
                    |c| matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9'),
                );
                tokens.push(Token::Identifier(text));
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

    token_tree::TokenTree::new(db, tokens, Span::from(start_pos, end_pos))
}

#[track_caller]
pub fn closing_delimeter(ch: char) -> char {
    match ch {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        _ => panic!("not a delimeter: {:?}", ch),
    }
}

fn accumulate(
    db: &dyn Lexer,
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
