#[macro_use]
pub mod origin_table;

pub mod class;
pub mod code;
pub mod diagnostic;
pub mod effect;
pub mod format_string;
pub mod function;
pub mod in_ir_db;
pub mod input_file;
pub mod intrinsic;
pub mod item;
pub mod kw;
pub mod lines;
pub mod parameter;
pub mod prelude;
pub mod return_type;
pub mod signature;
pub mod source_file;
pub mod span;
pub mod storage;
pub mod token;
pub mod token_tree;
pub mod word;

#[salsa::jar(db = Db)]
pub struct Jar(
    code::bir::Bir,
    code::syntax::Tree,
    code::validated::Tree,
    class::Class,
    diagnostic::Diagnostics,
    format_string::FormatString,
    format_string::FormatStringSection,
    function::Function,
    function::Variable,
    kw::Keywords,
    kw::keywords_map,
    lines::line_table,
    parameter::Parameter,
    source_file::SourceFile,
    token_tree::TokenTree,
    signature::Signature,
    signature::GenericParameters,
    signature::GenericParameter,
    signature::WhereClause,
    signature::Ty,
    signature::ParameterTy,
    signature::ClassTy,
    signature::Tys,
    signature::Permission,
    signature::KnownPermission,
    signature::Path,
    signature::Paths,
    word::Word,
    word::Words,
    return_type::ReturnType,
    input_file::InputFile,
);

pub trait Db: salsa::DbWithJar<Jar> {
    fn as_dyn_ir_db(&self) -> &dyn crate::Db;
}
impl<T: salsa::DbWithJar<Jar>> Db for T {
    fn as_dyn_ir_db(&self) -> &dyn crate::Db {
        self
    }
}
