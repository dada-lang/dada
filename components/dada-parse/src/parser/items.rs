use crate::{parser::Parser, token_test::SpannedIdentifier};

use dada_ir::{
    class::Class,
    code::{
        syntax::{op::Op, Spans, Tables},
        UnparsedCode,
    },
    effect::Effect,
    function::Function,
    item::Item,
    kw::Keyword,
    return_type::{ReturnType, ReturnTypeKind},
    source_file::SourceFile,
    span::Span,
};

use super::OrReportError;

impl<'db> Parser<'db> {
    pub(crate) fn parse_source_file(&mut self) -> SourceFile {
        let mut items = vec![];
        let mut exprs = vec![];
        let mut tables = Tables::default();
        let mut spans = Spans::default();
        while self.tokens.peek().is_some() {
            if let Some(item) = self.parse_item() {
                items.push(item);
            } else if let Some(expr) = self.parse_top_level_expr(&mut tables, &mut spans) {
                exprs.push(expr);
            } else {
                let span = self.tokens.last_span();
                self.tokens.consume();
                dada_ir::error!(span.in_file(self.filename), "unexpected token").emit(self.db);
            }
        }

        let syntax_tree = if !exprs.is_empty() {
            let start = spans[exprs[0]];
            Some(self.create_syntax_tree(start, vec![], tables, spans, exprs))
        } else {
            None
        };

        SourceFile::new(self.db, self.filename, items, syntax_tree)
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
        let code = UnparsedCode::new(parameter_tokens, body_tokens);
        let start_span = effect_span.unwrap_or(fn_span);
        Some(Function::new(
            self.db,
            func_name,
            effect,
            effect_span.unwrap_or(fn_span).in_file(self.filename),
            return_type,
            code,
            self.span_consumed_since(start_span).in_file(self.filename),
        ))
    }
}
