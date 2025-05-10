use std::pin::pin;

use dada_util::FromImpls;
use either::Either;

use crate::{
    check::{
        inference::{Direction, InferVarKind},
        red::{RedPerm, RedTy},
    },
    ir::{
        indices::{FromInfer, InferVarIndex},
        types::{SymGenericTerm, SymPerm, SymTy},
    },
};

use super::Env;

pub struct SymGenericTermBoundIterator<'db> {
    infer: InferVarIndex,
    kind: SymGenericTermBoundIteratorKind<'db>,
}

#[derive(FromImpls)]
enum SymGenericTermBoundIteratorKind<'db> {
    Ty(RedTyBoundIterator<'db>),
    Perm(RedPermBoundIterator<'db>),
}

impl<'db> SymGenericTermBoundIterator<'db> {
    pub fn new(env: &Env<'db>, infer: InferVarIndex, direction: Option<Direction>) -> Self {
        Self {
            infer,
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
                let perm = SymPerm::infer(db, env.perm_infer(self.infer));
                Some((direction, SymTy::perm(db, perm, sym_ty).into()))
            }
            SymGenericTermBoundIteratorKind::Perm(iter) => {
                let (direction, red_perm) = iter.next(env).await?;
                let sym_perm = red_perm.to_sym_perm(db);
                Some((direction, sym_perm.into()))
            }
        }
    }
}

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
