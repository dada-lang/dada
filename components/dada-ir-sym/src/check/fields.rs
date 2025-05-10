use dada_ir_ast::diagnostic::{Diagnostic, Err, Errors, Level, Reported};

use crate::{
    check::{env::Env, runtime::Runtime},
    ir::{
        binder::Binder, classes::SymField, populate::variable_decl_requires_default_perm,
        types::SymTy,
    },
};

use super::CheckTyInEnv;

/// Check the type of a field.
/// The returned type has two binders, the outer binder is the class,
/// the inner binder is the `self` place.
pub(crate) fn check_field<'db>(
    db: &'db dyn crate::Db,
    field: SymField<'db>,
) -> Errors<Binder<'db, Binder<'db, SymTy<'db>>>> {
    Runtime::execute(
        db,
        field.name_span(db),
        async move |runtime| -> Errors<Binder<'db, Binder<'db, SymTy<'db>>>> {
            let scope = field.into_scope(db);
            let mut env = Env::new(runtime, scope);

            let decl = field.source(db).variable(db);

            // In fields, we don't permit something like `x: String`,
            // user must write `x: my String`.
            if variable_decl_requires_default_perm(db, decl, &env.scope) {
                Diagnostic::new(
                    db,
                    Level::Error,
                    decl.base_ty(db).span(db),
                    "explicit permission required",
                )
                .report(db);
            }

            let ast_base_ty = decl.base_ty(db);
            let sym_base_ty = ast_base_ty.check_in_env(&mut env).await;
            let sym_ty = if let Some(ast_perm) = decl.perm(db) {
                let sym_perm = ast_perm.check_in_env(&mut env).await;
                SymTy::perm(db, sym_perm, sym_base_ty)
            } else {
                sym_base_ty
            };

            let bound_ty = env.into_scope().into_bound_value(db, sym_ty);
            Ok(bound_ty)
        },
        |bound_ty| bound_ty,
    )
}

pub(crate) fn field_err_ty<'db>(
    db: &'db dyn crate::Db,
    field: SymField<'db>,
    reported: Reported,
) -> Binder<'db, Binder<'db, SymTy<'db>>> {
    let scope = field.into_scope(db);
    scope.into_bound_value(db, SymTy::err(db, reported))
}
