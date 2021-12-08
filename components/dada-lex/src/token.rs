use dada_ast::word::Word;

use crate::token_tree;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    /// "foo"
    Identifier(Word),

    /// 22_000
    Number(Word),

    NewLine,

    /// An operator like `+` that is NOT followed by another operator.
    OpAlone(char),

    /// An operator like `+` that IS followed by another operator.
    OpAdjacent(char),
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    OpenBrace,
    CloseBrace,
    Tree(token_tree::TokenTree),
    Unknown(char),
}
