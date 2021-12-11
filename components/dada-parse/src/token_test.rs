use dada_ir::{kw::Keyword, token::Token, token_tree::TokenTree, word::Word};

/// Represents some kind of "condition test" that can be applied to tokens
/// (e.g., is an identifier or is a keyword).
pub(crate) trait TokenTest {
    /// When the test is successful, we return the token back but (potentially)
    /// with a narrower, more specific type -- this is that type.
    type Narrow;

    /// If `token` matches the condition, return `Some` with a potentially transformed
    /// version of the token. Else returns None.
    fn test(self, db: &dyn crate::Db, token: Token) -> Option<Self::Narrow>;
}

impl TokenTest for Keyword {
    type Narrow = Self;

    fn test(self, db: &dyn crate::Db, token: Token) -> Option<Self> {
        let Some(str) = token.alphabetic_str(db) else {
            return None;
        };

        if str == self.str() {
            Some(self)
        } else {
            None
        }
    }
}

/// A keyword like `class` or `async`
pub(crate) struct AnyKeyword;
impl TokenTest for AnyKeyword {
    type Narrow = Keyword;

    fn test(self, db: &dyn crate::Db, token: Token) -> Option<Keyword> {
        let word = token.alphabetic()?;
        dada_ir::kw::keywords(db).get(&word).copied()
    }
}

/// An `Alphabetic` that is not a keyword
pub(crate) struct Identifier;
impl TokenTest for Identifier {
    type Narrow = Word;

    fn test(self, db: &dyn crate::Db, token: Token) -> Option<Word> {
        let word = token.alphabetic()?;
        if dada_ir::kw::keywords(db).contains_key(&word) {
            None
        } else {
            Some(word)
        }
    }
}

impl TokenTest for Token {
    type Narrow = Token;

    fn test(self, _: &dyn crate::Db, token: Token) -> Option<Token> {
        if self == token {
            Some(token)
        } else {
            None
        }
    }
}

/// Any token at all
pub(crate) struct Any;
impl TokenTest for Any {
    type Narrow = Token;

    fn test(self, _: &dyn crate::Db, token: Token) -> Option<Token> {
        Some(token)
    }
}

/// Any token at all
pub(crate) struct AnyTree;
impl TokenTest for AnyTree {
    type Narrow = TokenTree;

    fn test(self, _: &dyn crate::Db, token: Token) -> Option<TokenTree> {
        token.tree()
    }
}
