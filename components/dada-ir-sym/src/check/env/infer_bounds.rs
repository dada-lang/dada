use std::pin::pin;

use dada_util::FromImpls;

use crate::{
    check::{
        inference::{Direction, InferVarKind},
        red::{RedPerm, RedTy},
    },
    ir::{
        indices::InferVarIndex,
        types::{SymGenericTerm, SymPerm},
    },
};

use super::Env;

/// Iterates over bounds on an inference variable in a given permission context.
/// Carries a permission that is applied to each bound as extract it.
pub struct SymGenericTermBoundIterator<'db> {
    /// The permission context in which this infer variable appears.
    /// This permission will eb applied to the bounds we extract.
    perm: SymPerm<'db>,

    /// A bounds iterator of suitable kind, depending on the kind of inference variable.
    kind: SymGenericTermBoundIteratorKind<'db>,
}

#[derive(FromImpls)]
enum SymGenericTermBoundIteratorKind<'db> {
    Ty(RedTyBoundIterator<'db>),
    Perm(RedPermBoundIterator<'db>),
}

impl<'db> SymGenericTermBoundIterator<'db> {
    pub fn new(
        env: &Env<'db>,
        perm: SymPerm<'db>,
        infer: InferVarIndex,
        direction: Option<Direction>,
    ) -> Self {
        Self {
            perm,
            kind: match env.infer_var_kind(infer) {
                InferVarKind::Type => RedTyBoundIterator::new(env, infer, direction).into(),
                InferVarKind::Perm => RedPermBoundIterator::new(env, infer, direction).into(),
            },
        }
    }

    pub async fn next(&mut self, env: &Env<'db>) -> Option<(Direction, SymGenericTerm<'db>)> {
        let db = env.db();
        match &mut self.kind {
            SymGenericTermBoundIteratorKind::Ty(iter) => {
                let (direction, red_ty) = iter.next(env).await?;
                let sym_ty = red_ty.to_sym_ty(db);
                Some((direction, self.perm.apply_to(db, sym_ty).into()))
            }
            SymGenericTermBoundIteratorKind::Perm(iter) => {
                let (direction, red_perm) = iter.next(env).await?;
                let sym_perm = red_perm.to_sym_perm(db);
                Some((direction, self.perm.apply_to(db, sym_perm).into()))
            }
        }
    }
}

/// Iterator over the red-ty bounds applied to `infer`
/// that are coming from a given direction (above/below).
pub struct RedTyBoundIterator<'db> {
    infer: InferVarIndex,
    direction: Option<Direction>,
    storage: [Option<Option<RedTy<'db>>>; 2],
}

impl<'db> RedTyBoundIterator<'db> {
    pub fn new(env: &Env<'db>, infer: InferVarIndex, direction: Option<Direction>) -> Self {
        assert_eq!(env.infer_var_kind(infer), InferVarKind::Type);
        Self {
            infer,
            direction,
            storage: [None, None],
        }
    }

    pub async fn next(&mut self, env: &Env<'db>) -> Option<(Direction, RedTy<'db>)> {
        match self.direction {
            None => {
                let [storage0, storage1] = &mut self.storage;
                futures::future::select(
                    pin!(Self::next_from_direction(
                        self.infer,
                        env,
                        Direction::FromAbove,
                        storage0,
                    )),
                    pin!(Self::next_from_direction(
                        self.infer,
                        env,
                        Direction::FromBelow,
                        storage1,
                    )),
                )
                .await
                .into_inner()
                .0
            }

            Some(direction) => {
                Self::next_from_direction(self.infer, env, direction, &mut self.storage[0]).await
            }
        }
    }

    async fn next_from_direction(
        infer: InferVarIndex,
        env: &Env<'db>,
        direction: Direction,
        storage: &mut Option<Option<RedTy<'db>>>,
    ) -> Option<(Direction, RedTy<'db>)> {
        loop {
            let bound = env
                .watch_inference_var(
                    infer,
                    |data| data.red_ty_bound(direction).map(|pair| pair.0),
                    storage,
                )
                .await;

            match bound {
                // Inference is complete.
                None => return None,

                // New bound value.
                Some(Some(bound)) => return Some((direction, bound)),

                // No current bound. Loop until one arrives.
                Some(None) => (),
            }
        }
    }
}

/// Iterator over the red-perm bounds applied to `infer`
/// that are coming from a given direction (above/below).
pub struct RedPermBoundIterator<'db> {
    infer: InferVarIndex,
    direction: Option<Direction>,
    storage: [Option<Option<RedPerm<'db>>>; 2],
}

impl<'db> RedPermBoundIterator<'db> {
    pub fn new(env: &Env<'db>, infer: InferVarIndex, direction: Option<Direction>) -> Self {
        assert_eq!(env.infer_var_kind(infer), InferVarKind::Perm);
        Self {
            infer,
            direction,
            storage: [None, None],
        }
    }

    pub async fn next(&mut self, env: &Env<'db>) -> Option<(Direction, RedPerm<'db>)> {
        match self.direction {
            None => {
                let [storage0, storage1] = &mut self.storage;
                futures::future::select(
                    pin!(Self::next_from_direction(
                        self.infer,
                        env,
                        Direction::FromAbove,
                        storage0,
                    )),
                    pin!(Self::next_from_direction(
                        self.infer,
                        env,
                        Direction::FromBelow,
                        storage1,
                    )),
                )
                .await
                .into_inner()
                .0
            }

            Some(direction) => {
                Self::next_from_direction(self.infer, env, direction, &mut self.storage[0]).await
            }
        }
    }

    async fn next_from_direction(
        infer: InferVarIndex,
        env: &Env<'db>,
        direction: Direction,
        storage: &mut Option<Option<RedPerm<'db>>>,
    ) -> Option<(Direction, RedPerm<'db>)> {
        loop {
            let bound = env
                .watch_inference_var(
                    infer,
                    |data| data.red_perm_bound(direction).map(|pair| pair.0),
                    storage,
                )
                .await;

            match bound {
                // Inference is complete.
                None => return None,

                // New bound value.
                Some(Some(bound)) => return Some((direction, bound)),

                // No current bound. Loop until one arrives.
                Some(None) => (),
            }
        }
    }
}
