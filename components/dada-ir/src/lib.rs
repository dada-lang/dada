#[macro_use]
pub mod origin_table;

pub mod class;
pub mod code;
pub mod diagnostic;
pub mod filename;
pub mod format_string;
pub mod func;
pub mod in_ir_db;
pub mod intrinsic;
pub mod item;
pub mod kw;
pub mod op;
pub mod parameter;
pub mod prelude;
pub mod span;
pub mod storage_mode;
pub mod token;
pub mod token_tree;
pub mod ty;
pub mod word;

#[salsa::jar(Db)]
pub struct Jar(
    code::syntax::Tree,
    code::validated::Tree,
    class::Class,
    diagnostic::Diagnostics,
    format_string::FormatString,
    format_string::FormatStringSection,
    func::Function,
    func::Variable,
    kw::keywords,
    op::binary_ops,
    parameter::Parameter,
    token_tree::TokenTree,
    ty::Ty,
    word::Word,
);

pub trait Db: salsa::DbWithJar<Jar> {}
impl<T: salsa::DbWithJar<Jar>> Db for T {}
