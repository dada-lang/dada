use crate::parser::Parser;

use dada_ir::{class::Class, function::Function, parameter::Parameter, token_tree::TokenTree};
use salsa::DebugWithDb;

#[salsa::memoized(in crate::Jar ref)]
#[allow(clippy::needless_lifetimes)]
pub fn parse_function_parameters(db: &dyn crate::Db, function: Function) -> Vec<Parameter> {
    if let Some(unparsed_code) = function.unparsed_code(db) {
        parse_parameters(db, unparsed_code.parameter_tokens)
    } else {
        panic!(
            "cannot parse parameters of function `{:?}` which did not have unparsed code",
            function.debug(db)
        )
    }
}

#[salsa::memoized(in crate::Jar ref)]
#[allow(clippy::needless_lifetimes)]
pub fn parse_class_parameters(db: &dyn crate::Db, class: Class) -> Vec<Parameter> {
    parse_parameters(db, class.field_tokens(db))
}

fn parse_parameters(db: &dyn crate::Db, token_tree: TokenTree) -> Vec<Parameter> {
    Parser::new(db, token_tree).parse_only_parameters()
}
