use dada_ir_ast::diagnostic::{Err, Errors, Reported};

use crate::{
    check::env::Env, check::runtime::Runtime, ir::binder::Binder, ir::classes::SymField,
    ir::types::SymTy,
};

use super::CheckInEnv;

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
            let env = Env::new(runtime, scope);
            let ast_ty = field.source(db).variable(db).ty(db);
            let ty = ast_ty.check_in_env(&env).await;
            let bound_ty = env.into_scope().into_bound_value(db, ty);
            Ok(bound_ty)
        },
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
