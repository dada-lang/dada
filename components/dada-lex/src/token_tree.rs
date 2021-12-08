use crate::{span::Span, token::Token, Jar};

salsa::entity2! {
    entity TokenTree in Jar {
        tokens: Vec<Token>,
        span: Span,
    }
}
