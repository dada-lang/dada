use crate::{
    filename::Filename,
    span::{FileSpan, Span},
    token::Token,
    Jar,
};

salsa::entity2! {
    entity TokenTree in Jar {
        filename: Filename,
        span: Span,
        #[value ref] tokens: Vec<Token>,
    }
}

impl<'db> salsa::DebugWithDb<'db> for TokenTree {
    type Db = dyn crate::Db + 'db;
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        let file_span: FileSpan = self.span(db).in_file(self.filename(db));
        write!(f, "Tokens({:?})", file_span.into_debug(db))
    }
}

impl TokenTree {
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
