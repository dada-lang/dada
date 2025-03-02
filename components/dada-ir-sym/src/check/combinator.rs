use std::pin::pin;

use dada_ir_ast::diagnostic::{Errors, Reported};
use futures::{
    StreamExt,
    future::{Either, LocalBoxFuture},
    stream::FuturesUnordered,
};

macro_rules! require_all {
    ($task0:expr, $task1:expr,) => {
        $crate::check::combinator::require_both($task0, $task1)
    };

    ($($task:expr,)*) => {
        $crate::check::combinator::require_all_(vec![
            $(Box::pin($task) as futures::future::LocalBoxFuture<'_, Errors<()>>),*
        ])
    };
}
pub(crate) use require_all;

use crate::ir::indices::InferVarIndex;

use super::{
    chains::{Chain, RedTy},
    env::Env,
    inference::InferenceVarData,
    report::ArcOrElse,
};

pub async fn require<'db>(
    a: impl Future<Output = Errors<bool>>,
    or_else: impl FnOnce() -> Reported,
) -> Errors<()> {
    if a.await? { Ok(()) } else { Err(or_else()) }
}

pub async fn require_for_all<'db, T>(
    items: impl IntoIterator<Item = T>,
    f: impl AsyncFn(T) -> Errors<()>,
) -> Errors<()> {
    let _v: Vec<()> = futures::future::try_join_all(items.into_iter().map(|elem| f(elem))).await?;
    Ok(())
}

pub async fn require_both<'db>(
    first: impl Future<Output = Errors<()>>,
    second: impl Future<Output = Errors<()>>,
) -> Errors<()> {
    let ((), ()) = futures::future::try_join(first, second).await?;
    Ok(())
}

pub async fn require_all_<'db>(
    tasks: impl IntoIterator<Item = LocalBoxFuture<'_, Errors<()>>>,
) -> Errors<()> {
    futures::future::try_join_all(tasks).await?;
    Ok(())
}

pub async fn not(a: impl Future<Output = Errors<bool>>) -> Errors<bool> {
    Ok(!a.await?)
}

pub async fn either(
    a: impl Future<Output = Errors<bool>>,
    b: impl Future<Output = Errors<bool>>,
) -> Errors<bool> {
    match futures::future::select(pin!(a), pin!(b)).await {
        Either::Left((Ok(true), _)) | Either::Right((Ok(true), _)) => Ok(true),
        Either::Left((Err(reported), _)) | Either::Right((Err(reported), _)) => Err(reported),
        Either::Left((Ok(false), f)) => f.await,
        Either::Right((Ok(false), f)) => f.await,
    }
}

/// Returns true if any of the items satisfies the predicate.
/// Returns false if not.
/// Stops executing as soon as either an error or a true result is found.
pub async fn for_all<'db, T>(
    items: impl IntoIterator<Item = T>,
    test_fn: impl AsyncFn(T) -> Errors<bool>,
) -> Errors<bool> {
    let mut unordered = FuturesUnordered::new();
    for item in items {
        unordered.push(test_fn(item));
    }
    let mut unordered = pin!(unordered);
    while let Some(r) = unordered.next().await {
        match r {
            Ok(true) => {}
            Ok(false) => return Ok(false),
            Err(reported) => return Err(reported),
        }
    }
    Ok(true)
}

/// Returns true if any of the items satisfies the predicate.
/// Returns false if not.
/// Stops executing as soon as either an error or a true result is found.
pub async fn exists<'db, T>(
    items: impl IntoIterator<Item = T>,
    test_fn: impl AsyncFn(T) -> Errors<bool>,
) -> Errors<bool> {
    let mut unordered = FuturesUnordered::new();
    for item in items {
        unordered.push(test_fn(item));
    }
    let mut unordered = pin!(unordered);
    while let Some(r) = unordered.next().await {
        match r {
            Ok(true) => return Ok(true),
            Ok(false) => {}
            Err(reported) => return Err(reported),
        }
    }
    Ok(false)
}

/// True if both `a` and `b` are true. Stops as soon as one is found to be false.
pub async fn both(
    a: impl Future<Output = Errors<bool>>,
    b: impl Future<Output = Errors<bool>>,
) -> Errors<bool> {
    match futures::future::select(pin!(a), pin!(b)).await {
        Either::Left((Ok(false), _)) | Either::Right((Ok(false), _)) => Ok(false),
        Either::Left((Err(reported), _)) | Either::Right((Err(reported), _)) => Err(reported),
        Either::Left((Ok(true), f)) => f.await,
        Either::Right((Ok(true), f)) => f.await,
    }
}

pub trait Extensions {
    /// Logically equivalent to `both(self, not(other))` but meant for the
    /// case where `self => !other`. Therefore, if `self` returns true,
    /// we return `true` immediately and if `other` returns true, we return `false` immediately.
    /// But if `other` returns false, we need to wait for `self` to complete.
    async fn and_not(self, other: impl Future<Output = Errors<bool>>) -> Errors<bool>;
}

impl<F> Extensions for F
where
    F: Future<Output = Errors<bool>>,
{
    async fn and_not(self, other: impl Future<Output = Errors<bool>>) -> Errors<bool> {
        match futures::future::select(pin!(self), pin!(other)).await {
            Either::Left((Err(reported), _)) | Either::Right((Err(reported), _)) => Err(reported),

            // If the LHS completed, we are done.
            // We could in theory wait for the RHS but it should be entailed by LHS.
            Either::Left((Ok(v), _)) => Ok(v),

            // If the RHS completed and was true, LHS cannot be true.
            Either::Right((Ok(true), _)) => Ok(false),

            // If the RHS completed and was false, that tells us nothing, need to wait for the LHS.
            Either::Right((Ok(false), f)) => f.await,
        }
    }
}

pub async fn exists_upper_bound<'db>(
    env: &Env<'db>,
    infer: InferVarIndex,
    mut op: impl AsyncFnMut(Chain<'db>) -> Errors<bool>,
) -> Errors<bool> {
    let mut observed = 0;
    let mut stack = vec![];

    loop {
        extract_bounding_chains(
            env,
            infer,
            &mut observed,
            &mut stack,
            &InferenceVarData::upper_chains,
        )
        .await;

        while let Some(chain) = stack.pop() {
            match op(chain).await {
                Ok(true) => return Ok(true),
                Ok(false) => (),
                Err(reported) => return Err(reported),
            }
        }
    }
}

/// Invoke `op` on every bounding chain (either upper or lower determined by `direction`).
/// Typically never returns as the full set of bounds on an inference variable is never known.
/// Exception is if an `Err` occurs, it is propagated.
pub async fn require_for_all_infer_bounds<'db>(
    env: &Env<'db>,
    infer: InferVarIndex,
    direction: impl for<'a> Fn(&'a InferenceVarData<'db>) -> &'a [(Chain<'db>, ArcOrElse<'db>)],
    mut op: impl AsyncFnMut(Chain<'db>) -> Errors<()>,
) -> Errors<()> {
    let mut observed = 0;
    let mut stack = vec![];

    loop {
        extract_bounding_chains(env, infer, &mut observed, &mut stack, &direction).await;

        while let Some(chain) = stack.pop() {
            match op(chain).await {
                Ok(()) => (),
                Err(reported) => return Err(reported),
            }
        }
    }
}

pub async fn require_for_all_infer_red_ty_bounds<'db>(
    env: &Env<'db>,
    infer: InferVarIndex,
    direction: impl for<'a> Fn(&'a InferenceVarData<'db>) -> &'a Option<(RedTy<'db>, ArcOrElse<'db>)>,
    mut op: impl AsyncFnMut(&RedTy<'db>) -> Errors<()>,
) -> Errors<()> {
    let mut red_ty = None;
    loop {
        red_ty = extract_red_ty(env, infer, red_ty, &direction).await;

        match &red_ty {
            Some(r) => op(r).await?,
            None => {
                // No further bounds, so `op` was true for all bounds.
                return Ok(());
            }
        }
    }
}

/// Monitor the inference variable `infer` and push new bounding chains (either upper or lower
/// depending on `direction`) onto `stack`. The variable `observed` is used to track which
/// chains have been observed from previous invocations; it should begin as `0` and it will be
/// incremented during the call.
pub async fn extract_bounding_chains<'db>(
    env: &Env<'db>,
    infer: InferVarIndex,
    observed: &mut usize,
    stack: &mut Vec<Chain<'db>>,
    direction: impl for<'a> Fn(&'a InferenceVarData<'db>) -> &'a [(Chain<'db>, ArcOrElse<'db>)],
) {
    env.runtime()
        .loop_on_inference_var(infer, |data| {
            let chains = direction(data);
            assert!(stack.is_empty());
            if *observed == chains.len() {
                None
            } else {
                stack.extend(chains.iter().skip(*observed).map(|pair| pair.0.clone()));
                *observed = chains.len();
                Some(())
            }
        })
        .await;
}

/// Monitor the red ty bounds on `infer`. Each time the fn is called, the result from any
/// previous call should be passed as `previous_red_ty`; pass `None` if the fn has never
/// been called before. This will return `Some(b)` if there is a new red ty bound on infer
/// or `None` if no further refined bounds are forthcoming.
pub async fn extract_red_ty<'db>(
    env: &Env<'db>,
    infer: InferVarIndex,
    previous_red_ty: Option<RedTy<'db>>,
    direction: &impl for<'a> Fn(&'a InferenceVarData<'db>) -> &'a Option<(RedTy<'db>, ArcOrElse<'db>)>,
) -> Option<RedTy<'db>> {
    env.runtime()
        .loop_on_inference_var(infer, |data| {
            let Some((next_red_ty, _or_else)) = direction(data) else {
                return None;
            };
            let next_red_ty = Some(next_red_ty.clone());
            if previous_red_ty == next_red_ty {
                None
            } else {
                next_red_ty
            }
        })
        .await
}
