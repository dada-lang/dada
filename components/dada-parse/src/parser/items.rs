use crate::{parser::Parser, token_test::Identifier};

use dada_ir::{
    class::Class,
    code::Code,
    func::{Effect, Function},
    item::Item,
    kw::Keyword,
};

use super::OrReportError;

impl<'db> Parser<'db> {
    pub(crate) fn parse_items(&mut self) -> Vec<Item> {
        let mut items = vec![];
        while self.tokens.peek().is_some() {
            if let Some(item) = self.parse_item() {
                items.push(item);
            } else {
                let span = self.tokens.last_span();
                self.tokens.consume();
                dada_ir::error!(span.in_file(self.filename), "unexpected token").emit(self.db);
            }
        }
        items
    }

    fn parse_item(&mut self) -> Option<Item> {
        if let Some(class) = self.parse_class() {
            Some(Item::Class(class))
        } else if let Some(function) = self.parse_function() {
            Some(Item::Function(function))
        } else {
            None
        }
    }

    fn parse_class(&mut self) -> Option<Class> {
        self.eat(Keyword::Class)?;
        let (class_name_span, class_name) = self
            .eat(Identifier)
            .or_report_error(self, || format!("expected a class name"))?;
        let (_, field_tokens) = self
            .delimited('(')
            .or_report_error(self, || format!("expected class parameters"))?;
        Some(Class::new(
            self.db,
            class_name,
            class_name_span,
            field_tokens,
        ))
    }

    fn parse_function(&mut self) -> Option<Function> {
        let async_kw = self.eat(Keyword::Async);
        let effect = if async_kw.is_some() {
            Effect::Async
        } else {
            Effect::None
        };
        self.eat(Keyword::Fn)
            .or_report_error(self, || format!("expected `fn`"))?;
        let (func_name_span, func_name) = self
            .eat(Identifier)
            .or_report_error(self, || format!("expected function name"))?;
        let (_, argument_tokens) = self
            .delimited('(')
            .or_report_error(self, || format!("expected function parameters"))?;
        let (_, body_tokens) = self
            .delimited('{')
            .or_report_error(self, || format!("expected function body"))?;
        let code = Code::new(self.db, body_tokens);
        Some(Function::new(
            self.db,
            func_name,
            func_name_span,
            effect,
            argument_tokens,
            code,
        ))
    }
}
