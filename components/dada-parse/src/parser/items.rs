use crate::{parser::Parser, token_test::Identifier};

use dada_ir::{
    class::Class,
    code::Code,
    func::{Effect, Function},
    item::Item,
    kw::Keyword,
    parameter::UnparsedParameters,
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
        } else {
            self.parse_function().map(Item::Function)
        }
    }

    fn parse_class(&mut self) -> Option<Class> {
        self.eat(Keyword::Class)?;
        let (class_name_span, class_name) = self
            .eat(Identifier)
            .or_report_error(self, || "expected a class name")?;
        let (_, field_tokens) = self
            .delimited('(')
            .or_report_error(self, || "expected class parameters")?;
        Some(Class::new(
            self.db,
            class_name,
            class_name_span.in_file(self.filename),
            UnparsedParameters(field_tokens),
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
            .or_report_error(self, || "expected `fn`".to_string())?;
        let (func_name_span, func_name) = self
            .eat(Identifier)
            .or_report_error(self, || "expected function name".to_string())?;
        let (_, parameter_tokens) = self
            .delimited('(')
            .or_report_error(self, || "expected function parameters".to_string())?;
        let (_, body_tokens) = self
            .delimited('{')
            .or_report_error(self, || "expected function body".to_string())?;
        let code = Code::new(body_tokens);
        Some(Function::new(
            self.db,
            func_name,
            func_name_span.in_file(self.filename),
            effect,
            UnparsedParameters(parameter_tokens),
            code,
        ))
    }
}
