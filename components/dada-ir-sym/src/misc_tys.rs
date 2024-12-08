use dada_ir_ast::diagnostic::{Err, Errors, Reported};

use crate::{binder::Binder, check::Runtime, class::SymField, env::Env, ty::SymTy};

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
            let ty = env.symbolize(ast_ty);
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
