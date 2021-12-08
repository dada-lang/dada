use dada_manifest::Text;

use crate::token_tree;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    Identifier(Text),
    Number(Text),
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    OpenBrace,
    CloseBrace,
    Tree(token_tree::TokenTree),
    Unknown(char),
}
