use dada_id::prelude::*;
use dada_ir::code::syntax;
use dada_ir::code::validated;
use dada_ir::code::Code;
use dada_ir::word::Word;

use super::name_lookup::LookupResult;

pub(crate) struct Validator<'me> {
    db: &'me dyn crate::Db,
    code: Code,
    syntax_tree: &'me syntax::Tree,
    tables: &'me mut validated::Tables,
    origins: &'me mut validated::Origins,
}

impl<'me> Validator<'me> {
    pub(crate) fn new(
        db: &'me dyn crate::Db,
        code: Code,
        syntax_tree: &'me syntax::Tree,
        tables: &'me mut validated::Tables,
        origins: &'me mut validated::Origins,
    ) -> Self {
        Self {
            db,
            code,
            syntax_tree,
            tables,
            origins,
        }
    }
    pub(crate) fn syntax_tables(&self) -> &'me syntax::Tables {
        &self.syntax_tree.tables
    }

    pub(crate) fn lookup(&self, name: Word) -> Option<LookupResult> {
        todo!()
    }

    pub(crate) fn validate_expr(&mut self, expr: syntax::Expr) -> validated::Expr {
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
