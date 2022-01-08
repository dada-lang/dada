use crate::parser::Parser;

use dada_ir::{
    parameter::{Parameter, UnparsedParameters},
    word::Word,
};

#[salsa::memoized(in crate::Jar ref)]
#[allow(clippy::needless_lifetimes)]
pub fn parse_parameters(db: &dyn crate::Db, parameters: UnparsedParameters) -> Vec<Parameter> {
    let token_tree = parameters.0;
    Parser::new(db, token_tree).parse_only_parameters()
}

#[salsa::memoized(in crate::Jar ref)]
pub fn parse_parameter_names(db: &dyn crate::Db, parameters: UnparsedParameters) -> Vec<Word> {
    parse_parameters(db, parameters)
        .iter()
        .map(|p| p.name(db))
        .collect()
}
