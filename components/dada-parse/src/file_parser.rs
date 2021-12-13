use crate::{parser::Parser, tokens::Tokens};

use dada_ir::{diagnostic::Diagnostic, item::Item, word::Word};

#[salsa::memoized(in crate::Jar ref)]
pub fn parse_file(db: &dyn crate::Db, filename: Word) -> (Vec<Item>, Vec<Diagnostic>) {
    let token_tree = dada_lex::lex_file(db, filename);
    let tokens = Tokens::new(db, token_tree);
    let mut parser = Parser::new(db, filename, tokens);
    let mut result = parser.parse_items();
    (result, parser.into_errors())
}

impl<'db> Parser<'db> {
    fn parse_items(&mut self) -> Vec<Item> {
        let mut items = vec![];
        while self.tokens.peek().is_some() {
            if let Some(item) = self.parse_item() {
                items.push(item);
            } else {
                let (span, _) = self.tokens.consume().unwrap();
                self.errors.push(Diagnostic {
                    filename: self.filename,
                    span,
                    message: format!("unexpected token"),
                });
            }
        }
        items
    }
}
