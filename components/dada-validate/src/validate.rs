use dada_id::prelude::*;
use dada_ir::code::syntax;
use dada_ir::code::validated;
use dada_ir::code::Code;
use dada_ir::word::Word;
use dada_parse::prelude::*;

#[salsa::memoized(in crate::Jar ref)]
pub fn validate_code(db: &dyn crate::Db, code: Code) -> validated::Tree {
    let syntax_tree = code.syntax_tree(db);
    let mut tables = validated::Tables::default();
    let mut spans = validated::Spans::default();
    let mut validator = Validator {
        db,
        syntax_tree,
        tables: &mut tables,
        spans: &mut spans,
    };
    let root_expr = validator.validate_expr(syntax_tree.root_expr);
    validated::Tree { tables, root_expr }
}

struct Validator<'me> {
    db: &'me dyn crate::Db,
    syntax_tree: &'me syntax::Tree,
    tables: &'me mut validated::Tables,
    spans: &'me mut validated::Spans,
}

impl<'me> Validator<'me> {
    fn syntax_tables(&self) -> &'me syntax::Tables {
        &self.syntax_tree.tables
    }

    fn validate_expr(&mut self, expr: syntax::Expr) -> validated::Expr {
        match expr.data(self.syntax_tables()) {
            syntax::ExprData::Id(name) => self.validate_id(expr, *name),
            syntax::ExprData::BooleanLiteral(_) => todo!(),
            syntax::ExprData::IntegerLiteral(_) => todo!(),
            syntax::ExprData::StringLiteral(_) => todo!(),
            syntax::ExprData::Dot(_, _) => todo!(),
            syntax::ExprData::Await(_) => todo!(),
            syntax::ExprData::Call(_, _) => todo!(),
            syntax::ExprData::Share(_) => todo!(),
            syntax::ExprData::Lease(_) => todo!(),
            syntax::ExprData::Give(_) => todo!(),
            syntax::ExprData::Var(_, _, _) => todo!(),
            syntax::ExprData::Parenthesized(_) => todo!(),
            syntax::ExprData::If(_, _, _) => todo!(),
            syntax::ExprData::Atomic(_) => todo!(),
            syntax::ExprData::Loop(_) => todo!(),
            syntax::ExprData::While(_, _) => todo!(),
            syntax::ExprData::Block(_) => todo!(),
            syntax::ExprData::Op(_, _, _) => todo!(),
            syntax::ExprData::OpEq(_, _, _) => todo!(),
            syntax::ExprData::Assign(_, _) => todo!(),
            syntax::ExprData::Error => todo!(),
        }
    }

    fn validate_id(&mut self, expr: syntax::Expr, name: Word) -> validated::Expr {
        todo!()
    }
}
