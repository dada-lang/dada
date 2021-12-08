use dada_ast::word::Word;

use crate::{token_tree, Lexer};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    /// "foo"
    Identifier(Word),

    /// 22_000
    Number(Word),

    /// An operator like `+` that is NOT followed by another operator.
    OpAlone(char),

    /// An operator like `+` that IS followed by another operator.
    OpAdjacent(char),

    /// `(`, `)`, `[`, `]`, `{`, or `}`
    Delimeter(char),

    /// When we encounter an opening delimeter, all the contents up to (but not including)
    /// the closing delimeter are read into a Tree.
    Tree(token_tree::TokenTree),

    /// Some whitespace (` `, `\n`, etc)
    Whitespace(char),

    /// Some unclassifiable, non-whitespace char
    Unknown(char),
}

impl Token {
    pub fn span_len(self, db: &dyn Lexer) -> u32 {
        match self {
            Token::Tree(tree) => tree.span(db).len(),
            Token::Identifier(word) | Token::Number(word) => {
                word.as_str(db).len().try_into().unwrap()
            }
            Token::Delimeter(ch)
            | Token::OpAlone(ch)
            | Token::OpAdjacent(ch)
            | Token::Whitespace(ch)
            | Token::Unknown(ch) => ch.len_utf8().try_into().unwrap(),
        }
    }
}
