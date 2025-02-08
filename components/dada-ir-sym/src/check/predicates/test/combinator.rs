use std::pin::pin;

use dada_ir_ast::diagnostic::{Errors, Reported};
use futures::future::Either;

pub async fn not(a: impl Future<Output = Errors<bool>>) -> Errors<bool> {
    Ok(!a.await?)
}
pub async fn either(
    a: impl Future<Output = Errors<bool>>,
    b: impl Future<Output = Errors<bool>>,
) -> Errors<bool> {
    let r = futures::future::try_join(async { looking_for_true(a.await) }, async {
        looking_for_true(b.await)
    })
    .await;

    match r {
        // Did not find true, so result is false
        Ok(((), ())) => Ok(false),

        // Found true, so result is true
        Err(None) => Ok(true),

        // Found an error
        Err(Some(reported)) => Err(reported),
    }
}

/// Returns true if any of the items satisfies the predicate.
/// Returns false if not.
/// Stops executing as soon as either an error or a true result is found.
pub async fn for_all<'db, T>(
    items: impl IntoIterator<Item = T>,
    test_fn: impl AsyncFn(T) -> Errors<bool>,
) -> Errors<bool> {
    let result = futures::future::try_join_all(
        items
            .into_iter()
            .map(|elem| async { looking_for_false(test_fn(elem).await) }),
    )
    .await;

    match result {
        // Did not find false, result is true
        Ok(_) => Ok(true),

        // Found false, result is false
        Err(None) => Ok(false),

        // Found an error
        Err(Some(reported)) => Err(reported),
    }
}

/// Returns true if any of the items satisfies the predicate.
/// Returns false if not.
/// Stops executing as soon as either an error or a true result is found.
pub async fn exists<'db, T>(
    items: impl IntoIterator<Item = T>,
    test_fn: impl AsyncFn(T) -> Errors<bool>,
) -> Errors<bool> {
    // Abuse `try_join_all` -- we return `Ok(())` if something returns false,
    // `Err(none)` if something returns true, and `Err(reported)` if an error is detected.
    // This way, execution ends as soon as we get some `Err` result.
    // We then report an error if `Ok(())` is returned, as that means that none of the tests were true.
    let result = futures::future::try_join_all(
        items
            .into_iter()
            .map(|elem| async { looking_for_true(test_fn(elem).await) }),
    )
    .await;

    match result {
        // Did not find true, result is false
        Ok(_v) => Ok(false),

        // Found true, result is true
        Err(None) => Ok(true),

        // Found an error
        Err(Some(reported)) => Err(reported),
    }
}

pub async fn both(
    a: impl Future<Output = Errors<bool>>,
    b: impl Future<Output = Errors<bool>>,
) -> Errors<bool> {
    let r = futures::future::try_join(async { looking_for_false(a.await) }, async {
        looking_for_false(b.await)
    })
    .await;

    match r {
        // Did not find false, so result is true
        Ok(((), ())) => Ok(true),

        // Found false, so result is false
        Err(None) => Ok(false),

        // Found an error
        Err(Some(reported)) => Err(reported),
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

pub async fn and_not(
    option1: impl AsyncFn() -> Errors<bool>,
    option2: impl AsyncFn() -> Errors<bool>,
) -> Errors<bool> {
    let r = futures::future::try_join(async { looking_for_true(option1().await) }, async {
        looking_for_false(option2().await)
    })
    .await;

    match r {
        // Did not find false, so result is true
        Ok(((), ())) => Ok(true),

        // Found false, so result is false
        Err(None) => Ok(false),

        // Found an error
        Err(Some(reported)) => Err(reported),
    }
}

type TheHunt = Result<(), Option<Reported>>;

fn looking_for_true(r: Errors<bool>) -> TheHunt {
    match r {
        Ok(true) => Err(None),
        Ok(false) => Ok(()),
        Err(reported) => Err(Some(reported)),
    }
}

fn looking_for_false(r: Errors<bool>) -> TheHunt {
    match r {
        Ok(false) => Err(None),
        Ok(true) => Ok(()),
        Err(reported) => Err(Some(reported)),
    }
}
