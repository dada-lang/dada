pub mod ast;
pub mod class;
pub mod span;
pub mod storage_mode;
pub mod ty;
pub mod word;

#[salsa::jar(Ast)]
pub struct Jar(class::Class, class::Field, word::Word, ty::Ty);

pub trait Ast: salsa::DbWithJar<Jar> {}
impl<T: salsa::DbWithJar<Jar>> Ast for T {}
