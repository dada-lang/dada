use crate::parser::Parser;

use dada_ir::code::{Ast, Block, BlockData, CodeTables, Expr};

impl Parser<'_> {
    pub(crate) fn parse_ast(&mut self) -> Ast {
        let mut code_parser = CodeParser {
            parser: self,
            tables: Default::default(),
        };

        let block = code_parser.parse_ast();
        Ast {
            tables: code_parser.tables,
            block,
        }
    }
}

struct CodeParser<'me, 'db> {
    parser: &'me mut Parser<'db>,
    tables: CodeTables,
}

impl CodeParser<'_, '_> {
    pub(crate) fn parse_ast(&mut self) -> Block {
        let mut exprs = vec![];
        while self.parser.tokens.peek().is_some() {
            let expr = self.parse_expr();
            exprs.push(expr);
        }
        self.tables.add(BlockData { exprs })
    }

    pub(crate) fn parse_expr(&mut self) -> Expr {
        todo!()
    }
}
