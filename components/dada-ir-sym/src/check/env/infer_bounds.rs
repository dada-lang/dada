use dada_util::FromImpls;
use serde::Serialize;

use crate::{
    check::{
        inference::{Direction, InferVarKind, InferenceVarData},
        red::{RedPerm, RedTy},
        report::ArcOrElse,
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

    /// Remember the inference variable
    infer: InferVarIndex,

    /// A bounds iterator of suitable kind, depending on the kind of inference variable.
    kind: SymGenericTermBoundIteratorKind<'db>,
}

#[derive(FromImpls)]
enum SymGenericTermBoundIteratorKind<'db> {
    Ty(RedTyBoundIterator<'db>),
    Perm(RedPermBoundIterator<'db>),
}

impl<'db> SymGenericTermBoundIterator<'db> {
    pub fn new(env: &Env<'db>, perm: SymPerm<'db>, infer: InferVarIndex) -> Self {
        Self {
            perm,
            infer,
            kind: match env.infer_var_kind(infer) {
                InferVarKind::Type => RedTyBoundIterator::new(env, infer).into(),
                InferVarKind::Perm => RedPermBoundIterator::new(env, infer).into(),
            },
        }
    }

    pub async fn next(&mut self, env: &Env<'db>) -> Option<(Direction, SymGenericTerm<'db>)> {
        let db = env.db();
        match &mut self.kind {
            SymGenericTermBoundIteratorKind::Ty(iter) => {
                let (direction, red_ty) = iter.next(env).await?;
                let sym_ty = red_ty.to_sym_ty(db);
                let result = self.perm.apply_to(db, sym_ty);
                env.log(
                    "next_bound",
                    &[&self.infer, &InferVarKind::Type, &direction, &result],
                );
                Some((direction, result.into()))
            }
            SymGenericTermBoundIteratorKind::Perm(iter) => {
                let (direction, red_perm) = iter.next(env).await?;
                let sym_perm = red_perm.to_sym_perm(db);
                let result = self.perm.apply_to(db, sym_perm);
                env.log(
                    "next_bound",
                    &[&self.infer, &InferVarKind::Perm, &direction, &result],
                );
                Some((direction, result.into()))
            }
        }
    }
}

/// Iterator over the red-ty bounds applied to `infer`
/// that are coming from a given direction (above/below).
pub struct RedTyBoundIterator<'db> {
    infer: InferVarIndex,
    storage: [Option<RedTy<'db>>; 2],
}

impl<'db> RedTyBoundIterator<'db> {
    pub fn new(env: &Env<'db>, infer: InferVarIndex) -> Self {
        assert_eq!(env.infer_var_kind(infer), InferVarKind::Type);
        Self {
            infer,
            storage: [None, None],
        }
    }

    #[track_caller]
    pub fn next(
        &mut self,
        env: &Env<'db>,
    ) -> impl Future<Output = Option<(Direction, RedTy<'db>)>> {
        env.log("next", &[&self.infer]);
        next_bound(
            env,
            self.infer,
            InferenceVarData::red_ty_bound,
            &mut self.storage,
        )
    }
}

/// Iterator over the red-perm bounds applied to `infer`
/// that are coming from a given direction (above/below).
pub struct RedPermBoundIterator<'db> {
    infer: InferVarIndex,
    storage: [Option<RedPerm<'db>>; 2],
}

impl<'db> RedPermBoundIterator<'db> {
    pub fn new(env: &Env<'db>, infer: InferVarIndex) -> Self {
        assert_eq!(env.infer_var_kind(infer), InferVarKind::Perm);
        Self {
            infer,
            storage: [None, None],
        }
    }

    #[track_caller]
    pub fn next(
        &mut self,
        env: &Env<'db>,
    ) -> impl Future<Output = Option<(Direction, RedPerm<'db>)>> {
        env.log("next", &[&self.infer]);
        next_bound(
            env,
            self.infer,
            InferenceVarData::red_perm_bound,
            &mut self.storage,
        )
    }
}

#[track_caller]
fn next_bound<'db, B>(
    env: &Env<'db>,
    infer: InferVarIndex,
    bound_op: impl Fn(&InferenceVarData<'db>, Direction) -> Option<(B, ArcOrElse<'db>)>,
    storage: &mut [Option<B>; 2],
) -> impl Future<Output = Option<(Direction, B)>>
where
    B: PartialEq + Serialize + 'db + Clone,
{
    env.loop_on_inference_var(infer, move |data| {
        if let Some((bound, _)) = bound_op(data, Direction::FromAbove)
            && Some(&bound) != storage[0].as_ref()
        {
            storage[0] = Some(bound.clone());
            Some((Direction::FromAbove, bound))
        } else if let Some((bound, _)) = bound_op(data, Direction::FromBelow)
            && Some(&bound) != storage[1].as_ref()
        {
            storage[1] = Some(bound.clone());
            Some((Direction::FromBelow, bound))
        } else {
            None
        }
    })
}
