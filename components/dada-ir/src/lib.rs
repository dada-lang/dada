pub mod class;
pub mod code;
pub mod func;
pub mod item;
pub mod kw;
pub mod span;
pub mod storage_mode;
pub mod token;
pub mod token_tree;
pub mod ty;
pub mod word;

#[salsa::jar(Db)]
pub struct Jar(
    code::Code,
    class::Class,
    class::Field,
    func::Function,
    func::Variable,
    word::Word,
    ty::Ty,
    token_tree::TokenTree,
    kw::keywords,
);

pub trait Db: salsa::DbWithJar<Jar> {}
impl<T: salsa::DbWithJar<Jar>> Db for T {}
