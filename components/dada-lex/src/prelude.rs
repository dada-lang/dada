use dada_ir::{
    kw,
    span::{FileSpan, Span},
    token::Token,
    token_tree::TokenTree,
};
use extension_trait::extension_trait;

#[extension_trait]
pub impl DadaLexFileSpanExt for FileSpan {
    /// Get the text for a span
    fn text(self, db: &dyn crate::Db) -> &str {
        let source_text = self.input_file.source_text(db);
        let start = usize::from(self.start);
        let end = usize::from(self.end);
        &source_text[start..end]
    }

    fn tokens(self, db: &dyn crate::Db) -> TokenTree {
        crate::lex::lex_filespan(db, self)
    }

    /// Return the span for the first use of `kw` within this span,
    /// or self if no use is found.
    fn leading_keyword(self, db: &dyn crate::Db, kw: kw::Keyword) -> FileSpan {
        let tokens = self.tokens(db).spanned_tokens(db);
        find_keyword(db, self, kw, tokens)
    }

    /// Return the span for the last use of `kw` within this span,
    /// or self if no use is found.
    fn trailing_keyword(self, db: &dyn crate::Db, kw: kw::Keyword) -> FileSpan {
        let mut tokens = self.tokens(db).spanned_tokens(db).collect::<Vec<_>>();
        tokens.reverse();
        find_keyword(db, self, kw, tokens)
    }
}

fn find_keyword(
    db: &dyn crate::Db,
    filespan: FileSpan,
    kw: kw::Keyword,
    tokens: impl IntoIterator<Item = (Span, Token)>,
) -> FileSpan {
    for (span, token) in tokens {
        match token {
            Token::Alphabetic(w) if w == kw.word(db) => {
                return span.anchor_to(db, filespan.input_file)
            }
            _ => continue,
        }
    }
    filespan
}
