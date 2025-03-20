use dada_ir_ast::diagnostic::Errors;
use dada_util::vecset::VecSet;

use crate::{
    check::{env::Env, inference::InferVarKind, red::{Chain, RedTerm, RedTy}, subtype::terms::require_sub_terms},
    ir::{indices::{FromInfer, InferVarIndex}, types::{SymTy, SymTyName}},
};


