use dada_ir_ast::{
    ast::{AstBlock, AstPath, AstStatement, BinaryOp, Literal, SpannedIdentifier},
    diagnostic::{Diagnostic, Level, Reported},
    span::{Span, Spanned},
};
use dada_util::FromImpls;
use salsa::Update;

use crate::{
    class::SymField,
    scope::{NameResolution, Resolve},
    symbol::SymLocalVariable,
    ir::ty::{SymGenericArg, SymPlace, SymPlaceKind, SymTy},
    IntoSymInScope,
};

