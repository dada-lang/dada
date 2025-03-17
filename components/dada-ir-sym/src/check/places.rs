use dada_ir_ast::diagnostic::Err;
use dada_util::boxed_async_fn;

use crate::{
    check::env::Env,
    ir::{
        classes::SymField,
        types::{SymGenericTerm, SymPerm, SymPlace, SymPlaceKind, SymTy},
    },
    prelude::CheckedFieldTy,
};

use super::{red::RedTy, to_red::ToRedTy};

pub trait PlaceTy<'db> {
    async fn place_ty(&self, env: &mut Env<'db>) -> SymTy<'db>;
}

impl<'db> PlaceTy<'db> for SymPlace<'db> {
    #[boxed_async_fn]
    async fn place_ty(&self, env: &mut Env<'db>) -> SymTy<'db> {
        match *self.kind(env.db()) {
            SymPlaceKind::Var(sym_variable) => env.variable_ty(sym_variable).await,
            SymPlaceKind::Field(owner_place, sym_field) => {
                let owner_ty = owner_place.place_ty(env).await;
                let (owner_red_ty, owner_perm) = owner_ty.to_red_ty(env);
                field_ty(env, owner_place, owner_perm, owner_red_ty, sym_field)
            }
            SymPlaceKind::Index(_sym_place) => {
                todo!()
            }
            SymPlaceKind::Error(reported) => SymTy::err(env.db(), reported),
            SymPlaceKind::Erased => panic!("cannot compute type of an erased place"),
        }
    }
}

fn field_ty<'db>(
    env: &mut Env<'db>,
    owner_place: SymPlace<'db>,
    owner_perm: Option<SymPerm<'db>>,
    owner_red_ty: RedTy<'db>,
    sym_field: SymField<'db>,
) -> SymTy<'db> {
    let db = env.db();
    match owner_red_ty {
        RedTy::Error(reported) => SymTy::err(db, reported),

        RedTy::Named(_name, generics) => {
            // FIXME: eventually we probably want to upcast here
            let field_ty = sym_field.checked_field_ty(db);
            let field_ty = field_ty
                .substitute(env.db(), &generics)
                .substitute(env.db(), &[SymGenericTerm::Place(owner_place)]);

            if let Some(owner_perm) = owner_perm {
                SymTy::perm(env.db(), owner_perm, field_ty)
            } else {
                field_ty
            }
        }

        RedTy::Infer(infer) => {
            // To have constructed this place there must have been a valid inference bound already
            let (infer_red_ty, _) = env
                .runtime()
                .with_inference_var_data(infer, |data| data.lower_red_ty())
                .unwrap();
            field_ty(env, owner_place, owner_perm, infer_red_ty, sym_field)
        }

        RedTy::Perm | RedTy::Var(_) | RedTy::Never => {
            unreachable!("no fields on a {owner_red_ty:?}")
        }
    }
}
