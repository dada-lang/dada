use dada_ir::{
    format_string::FormatString, kw::Keyword, token::Token, token_tree::TokenTree, word::Word,
};

/// Represents some kind of "condition test" that can be applied to a single token
/// (e.g., is an identifier or is a keyword).
pub(crate) trait TokenTest: std::fmt::Debug {
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
#[derive(Debug)]
pub(crate) struct AnyKeyword;
impl TokenTest for AnyKeyword {
    type Narrow = Keyword;

    fn test(self, db: &dyn crate::Db, token: Token) -> Option<Keyword> {
        let word = token.alphabetic()?;
        dada_ir::kw::keywords(db).get(&word).copied()
    }
}

/// An `Alphabetic` that is not a keyword
#[derive(Debug)]
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

/// A number like `22` or `22_000`.
///
/// Note that `.` is not accepted.
/// Floating point literals can be parsed by combining multiple tokens.
#[derive(Debug)]
pub(crate) struct Number;
impl TokenTest for Number {
    type Narrow = Word;

    fn test(self, _db: &dyn crate::Db, token: Token) -> Option<Word> {
        match token {
            Token::Number(w) => Some(w),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct FormatStringLiteral;
impl TokenTest for FormatStringLiteral {
    type Narrow = FormatString;

    fn test(self, _db: &dyn crate::Db, token: Token) -> Option<FormatString> {
        match token {
            Token::FormatString(fs) => Some(fs),
            _ => None,
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
#[derive(Debug)]
pub(crate) struct Any;
impl TokenTest for Any {
    type Narrow = Token;

    fn test(self, _: &dyn crate::Db, token: Token) -> Option<Token> {
        Some(token)
    }
}

/// Any token at all
#[derive(Debug)]
pub(crate) struct AnyTree;
impl TokenTest for AnyTree {
    type Narrow = TokenTree;

    fn test(self, _: &dyn crate::Db, token: Token) -> Option<TokenTree> {
        token.tree()
    }
}
