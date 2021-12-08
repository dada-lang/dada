use dada_ast::span::Span;

use crate::{token::Token, Jar};

salsa::entity2! {
    entity TokenTree in Jar {
        tokens: Vec<Token>,
        span: Span,
    }
}
