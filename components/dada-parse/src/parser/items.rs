//! Contains the methods to parse entire files or items.
//!
//! Does not parse the bodies of functions etc.

use crate::parser::Parser;

use dada_id::InternKey;
use dada_ir::{
    class::Class,
    code::{
        syntax::{self, Expr, ExprData, Spans, Tables, Tree, TreeData},
        UnparsedCode,
    },
    function::{Function, FunctionSignature},
    item::Item,
    kw::Keyword,
    source_file::{self, SourceFile},
    span::Span,
    word::Word,
};

use super::{CodeParser, Lookahead, OrReportError};

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
                dada_ir::error!(span.anchor_to(self.db, self.input_file), "unexpected token")
                    .emit(self.db);
            }
        }

        let main_fn = if !exprs.is_empty() {
            // Use the span of all the expression as the "span" of the main function -- this isn't
            // ideal, but it's ok for now. We should go through the cases below and find
            // the diagnostics, because they probably need some special casing to this
            // situation.
            let start_span = spans[exprs[0]];
            let end_span = spans[*exprs.last().unwrap()];
            let main_span = start_span.to(end_span);

            // Create the `main` function entity -- its code is already parsed, so use `None` for `unparsed_code`
            let main_name = Word::intern(self.db, source_file::TOP_LEVEL_FN);
            let function = Function::new(
                self.db,
                main_name,
                self.input_file,
                FunctionSignature::Main,
                None,
                main_span,
            );

            // Set the syntax-tree and parameters for the main function.
            let syntax_tree = self.create_syntax_tree(start_span, tables, spans, exprs);
            crate::code_parser::parse_function_body::specify(self.db, function, syntax_tree);

            items.push(Item::Function(function));
            Some(function)
        } else {
            None
        };

        SourceFile::new(self.db, self.input_file, items, main_fn)
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
        let start_span = self.peek_span();
        self.peek(Keyword::Class)?;

        let mut signature_tables = syntax::Tables::default();
        let mut signature_spans = syntax::Spans::default();
        let mut signature_parser = self.code_parser(&mut signature_tables, &mut signature_spans);

        let fn_decl = signature_parser.parse_class().unwrap();
        let name = signature_parser
            .parse_name()
            .or_report_error(&mut signature_parser, || "expected a class name")?;
        let generic_parameters = signature_parser.parse_generic_parameters();
        let parameters = signature_parser
            .parse_parameter_list()
            .or_report_error(&mut signature_parser, || "expected class parameters")?;
        let signature = syntax::Signature::new(
            name,
            fn_decl,
            None,
            generic_parameters,
            parameters,
            None,
            signature_tables,
            signature_spans,
        );

        Some(Class::new(
            self.db,
            name.data(&signature.tables).word,
            self.input_file,
            signature,
            self.span_consumed_since(start_span),
        ))
    }

    fn parse_function(&mut self) -> Option<Function> {
        // Look ahead to see if this is a function. It can look like
        //
        //     async? fn
        self.testahead(|this| {
            let _ = this.eat(Keyword::Async); // optional async keyword
            this.eat(Keyword::Fn).is_some()
        })?;

        let start_span = self.peek_span();

        let mut signature_tables = syntax::Tables::default();
        let mut signature_spans = syntax::Spans::default();
        let mut signature_parser = self.code_parser(&mut signature_tables, &mut signature_spans);
        let effect = signature_parser.parse_effect();
        let fn_kw = signature_parser.parse_fn().unwrap(); // we peeked above, it should be there
        let name = signature_parser
            .parse_name()
            .or_report_error(&mut signature_parser, || "expected function name")?;
        let generic_parameters = signature_parser.parse_generic_parameters();
        let parameters = signature_parser
            .parse_parameter_list()
            .or_report_error(&mut signature_parser, || {
                "expected function parameters".to_string()
            })?;
        let return_type = signature_parser.parse_return_type();
        let (_, body_tokens) = self
            .delimited('{')
            .or_report_error(self, || "expected function body")?;
        let code = UnparsedCode::new(body_tokens);
        let signature = syntax::Signature::new(
            name,
            fn_kw,
            effect,
            generic_parameters,
            parameters,
            return_type,
            signature_tables,
            signature_spans,
        );
        Some(Function::new(
            self.db,
            name.data(&signature.tables).word,
            self.input_file,
            FunctionSignature::Syntax(signature),
            Some(code),
            self.span_consumed_since(start_span),
        ))
    }

    pub(crate) fn parse_code_body(&mut self) -> Tree {
        let mut tables = Tables::default();
        let mut spans = Spans::default();

        let mut code_parser = CodeParser {
            parser: self,
            tables: &mut tables,
            spans: &mut spans,
        };

        let start = code_parser.tokens.last_span();
        let exprs = code_parser.parse_only_expr_seq();
        self.create_syntax_tree(start, tables, spans, exprs)
    }

    fn parse_top_level_expr(&mut self, tables: &mut Tables, spans: &mut Spans) -> Option<Expr> {
        let mut code_parser = CodeParser {
            parser: self,
            tables,
            spans,
        };
        code_parser.parse_expr()
    }

    fn create_syntax_tree(
        &mut self,
        start: Span,
        mut tables: Tables,
        mut spans: Spans,
        exprs: Vec<Expr>,
    ) -> Tree {
        let span = self.span_consumed_since(start);

        let root_expr = {
            let mut code_parser = CodeParser {
                parser: self,
                tables: &mut tables,
                spans: &mut spans,
            };
            code_parser.add(ExprData::Seq(exprs), span)
        };

        let tree_data = TreeData { root_expr };
        Tree::new(self.db, tree_data, tables, spans)
    }
}

impl CodeParser<'_, '_> {}
