#![allow(clippy::unused_unit)] // FIXME: salsa bug it seems

pub mod ast;
pub mod diagnostic;
pub mod inputs;
pub mod parse;
pub mod span;

#[salsa::jar(db = Db)]
pub struct Jar(
    ast::Module<'_>,
    ast::Identifier<'_>,
    ast::UseItem<'_>,
    ast::ClassItem<'_>,
    ast::Function<'_>,
    ast::FunctionBody<'_>,
    ast::AstTy<'_>,
    ast::AstPerm<'_>,
    ast::Literal<'_>,
    diagnostic::Diagnostics,
    inputs::SourceFile,
);

pub trait Db: salsa::DbWithJar<Jar> {}

impl<DB> Db for DB where DB: ?Sized + salsa::DbWithJar<Jar> {}
