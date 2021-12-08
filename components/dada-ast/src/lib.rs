pub mod ast;
pub mod span;
pub mod word;

#[salsa::jar(Ast)]
pub struct Jar(word::Word);

pub trait Ast: salsa::DbWithJar<Jar> {}
impl<T: salsa::DbWithJar<Jar>> Ast for T {}
