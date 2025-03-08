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

use super::{env::Env, inference::InferenceVarData, red::Chain, report::ArcOrElse};

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

pub async fn exists_infer_bound<'db>(
    env: &Env<'db>,
    infer: InferVarIndex,
    direction: impl for<'a> Fn(&'a InferenceVarData<'db>) -> &'a [(Chain<'db>, ArcOrElse<'db>)],
    mut op: impl AsyncFnMut(Chain<'db>) -> Errors<bool>,
) -> Errors<bool> {
    let mut observed = 0;
    let mut stack = vec![];

    loop {
        extract_bounding_chains(env, infer, &mut observed, &mut stack, &direction).await;

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
