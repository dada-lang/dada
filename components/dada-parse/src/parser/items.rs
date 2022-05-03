use crate::{parser::Parser, token_test::SpannedIdentifier};

use dada_ir::{
    class::Class,
    code::{syntax::op::Op, Code},
    effect::Effect,
    function::Function,
    item::Item,
    kw::Keyword,
    return_type::{ReturnType, ReturnTypeKind},
    span::Span,
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
        } else if let Some(func) = self.parse_function() {
            Some(Item::Function(func))
        } else {
            None
        }
    }

    fn parse_class(&mut self) -> Option<Class> {
        let (class_span, _) = self.eat(Keyword::Class)?;
        let (_, class_name) = self
            .eat(SpannedIdentifier)
            .or_report_error(self, || "expected a class name")?;
        let (_, field_tokens) = self
            .delimited('(')
            .or_report_error(self, || "expected class parameters")?;
        Some(Class::new(
            self.db,
            class_name,
            field_tokens,
            self.span_consumed_since(class_span).in_file(self.filename),
        ))
    }

    fn parse_function(&mut self) -> Option<Function> {
        let (effect_span, effect) = if let Some((span, _)) = self.eat(Keyword::Async) {
            (Some(span), Effect::Async)
        } else {
            (None, Effect::Default)
        };
        let (fn_span, _) = self
            .eat(Keyword::Fn)
            .or_report_error(self, || "expected `fn`".to_string())?;
        let (_, func_name) = self
            .eat(SpannedIdentifier)
            .or_report_error(self, || "expected function name".to_string())?;
        let (_, parameter_tokens) = self
            .delimited('(')
            .or_report_error(self, || "expected function parameters".to_string())?;
        let return_type = {
            let right_arrow = self.eat_op(Op::RightArrow);
            let span = right_arrow
                .unwrap_or_else(|| Span {
                    // span between last non skipped token and next non skippable token
                    start: self.tokens.last_span().end,
                    end: self.tokens.peek_span().start,
                })
                .in_file(self.filename);
            ReturnType::new(
                self.db,
                if right_arrow.is_some() {
                    ReturnTypeKind::Value
                } else {
                    ReturnTypeKind::Unit
                },
                span,
            )
        };
        let (_, body_tokens) = self
            .delimited('{')
            .or_report_error(self, || "expected function body".to_string())?;
        let code = Code::new(effect, Some(parameter_tokens), return_type, body_tokens);
        let start_span = effect_span.unwrap_or(fn_span);
        Some(Function::new(
            self.db,
            func_name,
            code,
            self.span_consumed_since(start_span).in_file(self.filename),
            effect_span.unwrap_or(fn_span).in_file(self.filename),
        ))
    }
}
