use crate::{token_test::*, tokens::Tokens};

use dada_ir::{
    class::Class,
    code::Code,
    diagnostic::Diagnostic,
    func::{Effect, Function},
    item::Item,
    kw::Keyword,
    span::Span,
    token::Token,
    token_tree::TokenTree,
    word::Word,
};

#[salsa::memoized(in crate::Jar ref)]
pub fn parse_file(db: &dyn crate::Db, filename: Word) -> (Vec<Item>, Vec<Diagnostic>) {
    let token_tree = dada_lex::lex_file(db, filename);
    let tokens = Tokens::new(db, token_tree);
    let mut parser = Parser {
        db,
        filename,
        tokens,
        result: vec![],
        errors: vec![],
    };
    parser.parse_items();
    (parser.result, parser.errors)
}

struct Parser<'db> {
    db: &'db dyn crate::Db,
    filename: Word,
    tokens: Tokens<'db>,
    result: Vec<Item>,
    errors: Vec<Diagnostic>,
}

impl<'db> Parser<'db> {
    /// Returns Some if the next pending token matches `is`, along
    /// with the narrowed view of the next token.
    fn peek_if<TT: TokenTest>(&mut self, is: TT) -> Option<TT::Narrow> {
        let token = self.tokens.peek()?;
        is.test(self.db, token)
    }

    /// If the next pending token matches `is`, consumes it and
    /// returns the span + narrowed view. Otherwise returns None
    /// and has no effect. Returns None if there is no pending token.
    fn eat_if<TT: TokenTest>(&mut self, is: TT) -> Option<(Span, TT::Narrow)> {
        let narrow = self.peek_if(is)?;
        self.tokens.consume();
        Some((self.tokens.last_span(), narrow))
    }

    fn parse_items(&mut self) {
        while self.tokens.peek().is_some() {
            if let Some(item) = self.parse_item() {
                self.result.push(item);
            } else {
                let (span, _) = self.tokens.consume().unwrap();
                self.errors.push(Diagnostic {
                    filename: self.filename,
                    span,
                    message: format!("unexpected token"),
                });
            }
        }
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
        self.eat_if(Keyword::Class)?;
        let (class_name_span, class_name) = self
            .eat_if(Identifier)
            .or_report_error(self, || format!("expected a class name"))?;
        let field_tokens = self
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
        let async_kw = self.eat_if(Keyword::Async);
        let effect = if async_kw.is_some() {
            Effect::Async
        } else {
            Effect::None
        };
        self.eat_if(Keyword::Fn)
            .or_report_error(self, || format!("expected `fn`"))?;
        let (func_name_span, func_name) = self
            .eat_if(Identifier)
            .or_report_error(self, || format!("expected function name"))?;
        let argument_tokens = self
            .delimited('(')
            .or_report_error(self, || format!("expected function parameters"))?;
        let body_tokens = self
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

    fn delimited(&mut self, delimiter: char) -> Option<TokenTree> {
        self.eat_if(Token::Delimiter(delimiter))?;

        // Lexer always produces a token tree as the next token after a delimiter:
        let (_, token_tree) = self.eat_if(AnyTree).unwrap();

        // Consume closing delimiter (if present)
        let closing_delimiter = dada_lex::closing_delimiter(delimiter);
        self.eat_if(Token::Delimiter(closing_delimiter))
            .or_report_error(self, || format!("expected `{closing_delimiter}`"));

        Some(token_tree)
    }
}

trait OrReportError {
    fn or_report_error(self, parser: &mut Parser<'_>, message: impl FnOnce() -> String) -> Self;
}

impl<T> OrReportError for Option<T> {
    fn or_report_error(self, parser: &mut Parser<'_>, message: impl FnOnce() -> String) -> Self {
        if self.is_some() {
            return self;
        }

        let span = parser.tokens.peek_span();
        parser.errors.push(Diagnostic {
            filename: parser.filename,
            span,
            message: message(),
        });
        None
    }
}
