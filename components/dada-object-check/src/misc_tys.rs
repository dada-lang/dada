use dada_ir_ast::diagnostic::Errors;
use dada_ir_sym::{binder::Binder, class::SymField};

use crate::{
    check::Runtime,
    env::Env,
    object_ir::{ObjectTy, ToObjectIr},
};

pub(crate) fn check_field<'db>(
    db: &'db dyn crate::Db,
    field: SymField<'db>,
) -> Errors<Binder<'db, Binder<'db, ObjectTy<'db>>>> {
    Runtime::execute(
        db,
        field.name_span(db),
        async move |runtime| -> Errors<Binder<'db, Binder<'db, ObjectTy<'db>>>> {
            let scope = field.into_scope(db);
            let env = Env::new(runtime, scope);
            let ty = field.ty(db).to_object_ir(&env);
            Ok(ty)
        },
    )
}
