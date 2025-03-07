use dada_ir_ast::diagnostic::Err;
use dada_util::boxed_async_fn;

use crate::{
    check::env::Env,
    ir::{
        classes::SymField,
        types::{SymGenericTerm, SymPlace, SymPlaceKind, SymTy, SymTyKind},
    },
    prelude::CheckedFieldTy,
};

pub trait PlaceTy<'db> {
    async fn place_ty(&self, env: &Env<'db>) -> SymTy<'db>;
}

impl<'db> PlaceTy<'db> for SymPlace<'db> {
    #[boxed_async_fn]
    async fn place_ty(&self, env: &Env<'db>) -> SymTy<'db> {
        match *self.kind(env.db()) {
            SymPlaceKind::Var(sym_variable) => env.variable_ty(sym_variable).await,
            SymPlaceKind::Field(sym_place, sym_field) => {
                let owner_ty = sym_place.place_ty(env).await;
                field_ty(env, *self, owner_ty, sym_field)
            }
            SymPlaceKind::Index(_sym_place) => {
                todo!()
            }
            SymPlaceKind::Error(reported) => SymTy::err(env.db(), reported),
        }
    }
}

fn field_ty<'db>(
    env: &Env<'db>,
    owner_place: SymPlace<'db>,
    owner_ty: SymTy<'db>,
    sym_field: SymField<'db>,
) -> SymTy<'db> {
    match *owner_ty.kind(env.db()) {
        SymTyKind::Perm(sym_perm, sym_ty) => {
            let field_ty = field_ty(env, owner_place, sym_ty, sym_field);
            SymTy::perm(env.db(), sym_perm, field_ty)
        }
        SymTyKind::Named(_name, ref generics) => {
            // FIXME: eventually we probably want to upcast here
            let field_ty = sym_field.checked_field_ty(env.db());
            field_ty
                .substitute(env.db(), &generics)
                .substitute(env.db(), &[SymGenericTerm::Place(owner_place)])
        }
        SymTyKind::Infer(_) => todo!(),
        SymTyKind::Var(_) | SymTyKind::Never => unreachable!("no fields on these types"),
        SymTyKind::Error(reported) => SymTy::err(env.db(), reported),
    }
}
