use crate::{
    input_file::InputFile,
    span::{FileSpan, Span},
    token::Token,
};

#[salsa::tracked]
pub struct TokenTree {
    input_file: InputFile,
    span: Span,
    #[return_ref]
    tokens: Vec<Token>,
}

impl<Db: ?Sized + crate::Db> salsa::DebugWithDb<Db> for TokenTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        let db = db.as_dyn_ir_db();
        let file_span: FileSpan = self.span(db).in_file(self.input_file(db));
        write!(f, "Tokens({:?})", file_span.into_debug(db))
    }
}

impl TokenTree {
    #[allow(clippy::len_without_is_empty)]
    pub fn len(self, db: &dyn crate::Db) -> u32 {
        self.span(db).len()
    }

    pub fn spanned_tokens(self, db: &dyn crate::Db) -> impl Iterator<Item = (Span, Token)> + '_ {
        let mut start = self.span(db).start;
        self.tokens(db).iter().map(move |token| {
            let len = token.span_len(db);
            let span = Span::from(start, start + len);
            start = start + len;
            (span, *token)
        })
    }
}
