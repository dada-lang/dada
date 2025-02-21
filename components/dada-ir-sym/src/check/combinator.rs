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
        $crate::check::combinator::require_all(vec![
            $(Box::pin($task) as futures::future::LocalBoxFuture<'_, Errors<()>>),*
        ])
    };
}
pub(crate) use require_all;

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

pub async fn require_all<'db>(
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
