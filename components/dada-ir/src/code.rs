use crate::{effect::Effect, filename::Filename, span::FileSpan, token_tree::TokenTree};

/// "Code" represents a block of code attached to a method.
/// After parsing, it just contains a token tree, but you can...
///
/// * use the `ast` method from the `dada_parse` prelude to
///   parse it into an `Ast`.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Code {
    /// Declared effect for the function body -- e.g., `async fn` would have
    /// this be `async`. This can affect validation and code generation.
    pub effect: Effect,

    /// Tokens for the parameter list (parsed when we generate the syntax tree).
    pub parameter_tokens: Option<TokenTree>,

    /// Span of `->` symbol if it has one.
    pub return_type: Option<FileSpan>,

    /// Tokens for the body (parsed when we generate the syntax tree).
    pub body_tokens: TokenTree,
}

impl Code {
    pub fn new(
        effect: Effect,
        parameter_tokens: Option<TokenTree>,
        return_type: Option<FileSpan>,
        body_tokens: TokenTree,
    ) -> Self {
        Self {
            effect,
            parameter_tokens,
            return_type,
            body_tokens,
        }
    }

    pub fn filename(self, db: &dyn crate::Db) -> Filename {
        self.body_tokens.filename(db)
    }
}

impl<Db: ?Sized + crate::Db> salsa::DebugWithDb<Db> for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        f.debug_struct("Code")
            .field("parameter_tokens", &self.parameter_tokens.debug(db))
            .field("return_type", &self.return_type.debug(db))
            .field("body_tokens", &self.body_tokens.debug(db))
            .finish()
    }
}

pub mod bir;
pub mod syntax;
pub mod validated;
