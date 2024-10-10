use dada_ir_sym::{
    symbol::{SymGeneric, SymGenericKind, SymLocalVariable},
    ty::{Binder, SymGenericTerm, SymPerm, SymTy},
};
use dada_util::Map;

use crate::{executor::Check, universe::Universe};

#[derive(Clone)]
pub struct Env<'db> {
    universe: Universe,
    generic_variables: Vec<SymGeneric<'db>>,
    program_variables: Map<SymLocalVariable<'db>, SymTy<'db>>,
}

impl<'db> Env<'db> {
    /// Create an empty environment
    pub fn new() -> Self {
        Self {
            universe: Universe::ROOT,
            program_variables: Map::default(),
        }
    }

    pub fn universe(&self) -> Universe {
        self.universe
    }

    pub fn open_in_next_universe<T: Update>(&mut self, binder: Binder<T>) {}

    // Modify this environment to put it in a new universe.
    pub fn increment_universe(&mut self) {
        self.universe = self.universe.next();
    }

    pub fn insert_program_variable(&mut self, lv: SymLocalVariable<'db>, ty: SymTy<'db>) {
        self.program_variables.insert(lv, ty);
    }

    pub fn fresh_inference_var(
        &self,
        check: &mut Check<'_, 'db>,
        kind: SymGenericKind,
    ) -> SymGenericTerm<'db> {
        check.fresh_inference_var(SymGenericKind::Perm, self.universe)
    }

    pub fn fresh_ty_inference_var(&self, check: &mut Check<'_, 'db>) -> SymTy<'db> {
        let SymGenericTerm::Type(ty) = self.fresh_inference_var(check, SymGenericKind::Type) else {
            unreachable!();
        };
        ty
    }

    pub fn fresh_perm_inference_var(&self, check: &mut Check<'_, 'db>) -> SymPerm<'db> {
        let SymGenericTerm::Perm(perm) = self.fresh_inference_var(check, SymGenericKind::Perm)
        else {
            unreachable!();
        };
        perm
    }
}
