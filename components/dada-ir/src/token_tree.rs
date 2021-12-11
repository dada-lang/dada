use crate::{span::Span, token::Token, Jar};

salsa::entity2! {
    entity TokenTree in Jar {
        #[value ref] tokens: Vec<Token>,
        span: Span,
    }
}

impl TokenTree {
    pub fn spanned_tokens(self, db: &dyn crate::Db) -> impl Iterator<Item = (Span, Token)> + '_ {
        let mut start = self.span(db).start;
        self.tokens(db).iter().map(move |token| {
            let len = token.span_len(db);
            let span = Span::from(start, start + len);
            start += len;
            (span, *token)
        })
    }
}
