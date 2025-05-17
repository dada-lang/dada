use dada_ir_ast::diagnostic::Errors;

use crate::{
    check::{env::Env, report::OrElse},
    ir::generics::{SymWhereClause, SymWhereClauseKind},
};

use super::{
    require_lent::require_term_is_lent, require_owned::require_term_is_owned,
    require_shared::require_term_is_shared, require_unique::require_term_is_unique,
};

pub async fn require_where_clause<'db>(
    env: &mut Env<'db>,
    where_clause: SymWhereClause<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let db = env.db();
    let subject = where_clause.subject(db);
    match where_clause.kind(db) {
        SymWhereClauseKind::Unique => require_term_is_unique(env, subject, or_else).await,
        SymWhereClauseKind::Shared => require_term_is_shared(env, subject, or_else).await,
        SymWhereClauseKind::Owned => require_term_is_owned(env, subject, or_else).await,
        SymWhereClauseKind::Lent => require_term_is_lent(env, subject, or_else).await,
    }
}
