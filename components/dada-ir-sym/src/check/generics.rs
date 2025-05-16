use dada_ir_ast::ast::{AstWhereClause, AstWhereClauseKind};

use crate::ir::generics::{SymWhereClause, SymWhereClauseKind};

use super::{CheckTyInEnv, env::Env};

pub async fn symbolify_ast_where_clause<'db>(
    env: &mut Env<'db>,
    ast_where_clause: AstWhereClause<'db>,
    output: &mut Vec<SymWhereClause<'db>>,
) {
    let db = env.db();
    let subject = ast_where_clause.subject(db).check_in_env(env).await;
    let mut push_kind =
        |kind: SymWhereClauseKind| output.push(SymWhereClause::new(db, subject, kind));

    for kind in ast_where_clause.kinds(db) {
        match kind {
            AstWhereClauseKind::Reference(_) => {
                push_kind(SymWhereClauseKind::Shared);
                push_kind(SymWhereClauseKind::Lent);
            }
            AstWhereClauseKind::Mutable(_) => {
                push_kind(SymWhereClauseKind::Unique);
                push_kind(SymWhereClauseKind::Lent);
            }
            AstWhereClauseKind::Shared(_) => {
                push_kind(SymWhereClauseKind::Shared);
            }
            AstWhereClauseKind::Owned(_) => {
                push_kind(SymWhereClauseKind::Owned);
            }
            AstWhereClauseKind::Lent(_) => {
                push_kind(SymWhereClauseKind::Lent);
            }
            AstWhereClauseKind::Unique(_) => {
                push_kind(SymWhereClauseKind::Unique);
            }
        }
    }
}
