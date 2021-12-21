use crate::parser::Parser;

use dada_ir::parameter::{Parameter, UnparsedParameters};

#[salsa::memoized(in crate::Jar ref)]
pub fn parse_parameters(db: &dyn crate::Db, parameters: UnparsedParameters) -> Vec<Parameter> {
    let token_tree = parameters.0;
    Parser::new(db, token_tree).parse_only_parameters()
}
