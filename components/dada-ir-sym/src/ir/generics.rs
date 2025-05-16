use dada_util::SalsaSerialize;
use salsa::Update;
use serde::Serialize;

use super::types::SymGenericTerm;

#[derive(SalsaSerialize)]
#[salsa::interned(debug)]
pub struct SymWhereClause<'db> {
    pub subject: SymGenericTerm<'db>,
    pub kind: SymWhereClauseKind,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, Serialize)]
pub enum SymWhereClauseKind {
    Unique,
    Shared,
    Owned,
    Lent,
}

