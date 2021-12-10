pub mod class;
pub mod code;
pub mod func;
pub mod span;
pub mod storage_mode;
pub mod ty;
pub mod word;

#[salsa::jar(Ast)]
pub struct Jar(
    code::Code,
    class::Class,
    class::Field,
    func::Func,
    func::Variable,
    word::Word,
    ty::Ty,
);

pub trait Ast: salsa::DbWithJar<Jar> {}
impl<T: salsa::DbWithJar<Jar>> Ast for T {}
