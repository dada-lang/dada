use std::{panic::Location, pin::pin, task::Poll};

use dada_ir_ast::diagnostic::{Errors, Reported};
use futures::{
    FutureExt, StreamExt,
    future::{Either, LocalBoxFuture},
    stream::FuturesUnordered,
};
use serde::Serialize;

use crate::{
    check::{alternatives::Alternative, debug::TaskDescription, inference::Direction, red::RedTy},
    ir::indices::InferVarIndex,
};

use crate::check::{env::Env, inference::InferenceVarData, red::RedPerm, report::ArcOrElse};

impl<'db> Env<'db> {
    pub async fn require(
        &mut self,
        a: impl AsyncFnOnce(&mut Env<'db>) -> Errors<bool>,
        or_else: impl FnOnce(&mut Env<'db>) -> Reported,
    ) -> Errors<()> {
        if a(self).await? {
            Ok(())
        } else {
            Err(or_else(self))
        }
    }

    #[track_caller]
    pub fn require_for_all<T>(
        &mut self,
        items: impl IntoIterator<Item = T>,
        f: impl AsyncFn(&mut Env<'db>, T) -> Errors<()>,
    ) -> impl Future<Output = Errors<()>> {
        let caller = Location::caller();
        async move {
            let this = &*self;
            let f = &f;
            let _v: Vec<()> =
                futures::future::try_join_all(items.into_iter().zip(0..).map(|(elem, index)| {
                    let mut env =
                        this.fork(|handle| handle.spawn(caller, TaskDescription::Require(index)));
                    async move { f(&mut env, elem).await }
                }))
                .await?;
            Ok(())
        }
    }

    pub fn require_all(&mut self) -> RequireAll<'_, 'db> {
        RequireAll {
            env: self,
            required: vec![],
        }
    }

    #[track_caller]
    pub fn require_both(
        &mut self,
        a: impl AsyncFnOnce(&mut Self) -> Errors<()>,
        b: impl AsyncFnOnce(&mut Self) -> Errors<()>,
    ) -> impl Future<Output = Errors<()>> {
        let caller = Location::caller();
        async move {
            let ((), ()) = futures::future::try_join(
                async {
                    let mut env =
                        self.fork(|handle| handle.spawn(caller, TaskDescription::Require(0)));
                    let result = a(&mut env).await;
                    env.log_result(caller, result)
                },
                async {
                    let mut env =
                        self.fork(|handle| handle.spawn(caller, TaskDescription::Require(1)));
                    let result = b(&mut env).await;
                    env.log_result(caller, result)
                },
            )
            .await?;
            Ok(())
        }
    }

    #[track_caller]
    pub fn join<A, B>(
        &mut self,
        a: impl AsyncFnOnce(&mut Self) -> A,
        b: impl AsyncFnOnce(&mut Self) -> B,
    ) -> impl Future<Output = (A, B)>
    where
        A: erased_serde::Serialize,
        B: erased_serde::Serialize,
    {
        let caller = Location::caller();
        futures::future::join(
            async {
                let mut env = self.fork(|handle| handle.spawn(caller, TaskDescription::Join(0)));
                let result = a(&mut env).await;
                env.log_result(caller, result)
            },
            async {
                let mut env = self.fork(|handle| handle.spawn(caller, TaskDescription::Join(1)));
                let result = b(&mut env).await;
                env.log_result(caller, result)
            },
        )
    }

    #[track_caller]
    pub fn either(
        &mut self,
        a: impl AsyncFnOnce(&mut Env<'db>) -> Errors<bool>,
        b: impl AsyncFnOnce(&mut Env<'db>) -> Errors<bool>,
    ) -> impl Future<Output = Errors<bool>> {
        let caller = Location::caller();

        async move {
            let a = pin!(async {
                let mut env = self.fork(|handle| handle.spawn(caller, TaskDescription::Any(0)));
                let result = a(&mut env).await;
                env.log_result(caller, result)
            });

            let b = pin!(async {
                let mut env = self.fork(|handle| handle.spawn(caller, TaskDescription::Any(1)));
                let result = b(&mut env).await;
                env.log_result(caller, result)
            });

            match futures::future::select(a, b).await {
                Either::Left((Ok(true), _)) | Either::Right((Ok(true), _)) => Ok(true),
                Either::Left((Err(reported), _)) | Either::Right((Err(reported), _)) => {
                    Err(reported)
                }
                Either::Left((Ok(false), f)) => f.await,
                Either::Right((Ok(false), f)) => f.await,
            }
        }
    }

    /// Returns true if any of the items satisfies the predicate.
    /// Returns false if not.
    /// Stops executing as soon as either an error or a true result is found.
    #[track_caller]
    pub fn for_all<T>(
        &mut self,
        items: impl IntoIterator<Item = T>,
        test_fn: impl AsyncFn(&mut Env<'db>, T) -> Errors<bool>,
    ) -> impl Future<Output = Errors<bool>> {
        let compiler_location = Location::caller();

        async move {
            let this = &*self;
            let test_fn = &test_fn;
            let mut unordered = FuturesUnordered::new();
            for (item, index) in items.into_iter().zip(0..) {
                unordered.push(async move {
                    let mut env = this.fork(|handle| {
                        handle.spawn(compiler_location, TaskDescription::All(index))
                    });
                    let result = test_fn(&mut env, item).await;
                    env.log_result(compiler_location, result)
                });
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
    }

    /// Returns true if any of the items satisfies the predicate.
    /// Returns false if not.
    /// Stops executing as soon as either an error or a true result is d.
    #[track_caller]
    pub fn exists<T>(
        &mut self,
        items: impl IntoIterator<Item = T>,
        test_fn: impl AsyncFn(&mut Env<'db>, T) -> Errors<bool>,
    ) -> impl Future<Output = Errors<bool>> {
        let compiler_location = Location::caller();

        async move {
            let this = &*self;
            let test_fn = &test_fn;
            let mut unordered = FuturesUnordered::new();
            for (item, index) in items.into_iter().zip(0..) {
                unordered.push(async move {
                    let mut env = this.fork(|handle| {
                        handle.spawn(compiler_location, TaskDescription::Any(index))
                    });
                    let result = test_fn(&mut env, item).await;
                    env.log_result(compiler_location, result)
                });
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
    }

    /// True if both `a` and `b` are true. Stops as soon as one is found to be false.
    #[track_caller]
    pub fn both(
        &mut self,
        a: impl AsyncFnOnce(&mut Env<'db>) -> Errors<bool>,
        b: impl AsyncFnOnce(&mut Env<'db>) -> Errors<bool>,
    ) -> impl Future<Output = Errors<bool>> {
        let compiler_location = Location::caller();

        async move {
            let a = async {
                let mut env =
                    self.fork(|handle| handle.spawn(compiler_location, TaskDescription::All(0)));
                let result = a(&mut env).await;
                env.log_result(compiler_location, result)
            };

            let b = async {
                let mut env =
                    self.fork(|handle| handle.spawn(compiler_location, TaskDescription::All(1)));
                let result = b(&mut env).await;
                env.log_result(compiler_location, result)
            };

            match futures::future::select(pin!(a), pin!(b)).await {
                Either::Left((Ok(false), _)) | Either::Right((Ok(false), _)) => Ok(false),
                Either::Left((Err(reported), _)) | Either::Right((Err(reported), _)) => {
                    Err(reported)
                }
                Either::Left((Ok(true), f)) => f.await,
                Either::Right((Ok(true), f)) => f.await,
            }
        }
    }

    /// Invoke `op` each time `infer`'s bounds get tighter.
    /// If `op` returns `Some`, return that.
    /// If `op` returns `None`, wait until tighter bounds arrive.
    /// Returns `None` if inference completes before finding a bound for which `op`
    /// returns `Some`.
    #[track_caller]
    pub fn find_red_perm_bound(
        &mut self,
        infer: InferVarIndex,
        direction: Option<Direction>,
        mut op: impl AsyncFnMut(&mut Env<'db>, Direction, RedPerm<'db>) -> Errors<Option<bool>>,
    ) -> impl Future<Output = Errors<Option<bool>>> {
        let compiler_location = Location::caller();

        async move {
            self.indent_with_compiler_location(
                compiler_location,
                "find_red_perm_bound",
                &[&infer],
                async |env| {
                    let mut storage = None;
                    while let Some((d, b)) =
                        env.next_perm_bound(infer, direction, &mut storage).await
                    {
                        match op(env, d, b).await? {
                            Some(b) => return Ok(Some(b)),
                            None => (),
                        }
                    }
                    Ok(None)
                },
            )
            .await
        }
    }

    /// Observe the red-perm bound on `infer` from the given direction
    /// and invoke `op` each time it changes. If `op` ever results in an
    /// error, propagate it.
    #[track_caller]
    pub fn require_for_all_red_perm_bounds(
        &mut self,
        infer: InferVarIndex,
        direction: Option<Direction>,
        mut op: impl AsyncFnMut(&mut Env<'db>, Direction, RedPerm<'db>) -> Errors<()>,
    ) -> impl Future<Output = Errors<()>> {
        let compiler_location = Location::caller();

        async move {
            self.indent_with_compiler_location(
                compiler_location,
                "require_for_all_red_perm_bounds",
                &[&infer],
                async |env| {
                    let mut storage = None;
                    while let Some((d, b)) =
                        env.next_perm_bound(infer, direction, &mut storage).await
                    {
                        op(env, d, b).await?;
                    }
                    Ok(())
                },
            )
            .await
        }
    }

    #[track_caller]
    pub async fn next_perm_bound(
        &self,
        infer: InferVarIndex,
        direction: Option<Direction>,
        storage: &mut Option<Option<(Direction, RedPerm<'db>)>>,
    ) -> Option<(Direction, RedPerm<'db>)> {
        loop {
            let bound = self
                .watch_inference_var(
                    infer,
                    |data| {
                        let from_above = || {
                            data.red_perm_bound(Direction::FromAbove)
                                .map(|pair| (Direction::FromAbove, pair.0))
                        };

                        let from_below = || {
                            data.red_perm_bound(Direction::FromBelow)
                                .map(|pair| (Direction::FromBelow, pair.0))
                        };

                        match direction {
                            Some(Direction::FromAbove) => from_above(),
                            Some(Direction::FromBelow) => from_below(),
                            None => from_above().or_else(from_below),
                        }
                    },
                    storage,
                )
                .await;

            match bound {
                // Inference is complete.
                None => return None,

                // New bound value.
                Some(Some(b)) => return Some(b),

                // No current bound. Loop until one arrives.
                Some(None) => (),
            }
        }
    }

    #[track_caller]
    pub fn loop_on_inference_var<T>(
        &self,
        infer: InferVarIndex,
        op: impl FnMut(&InferenceVarData<'db>) -> Option<T>,
    ) -> impl Future<Output = Option<T>>
    where
        T: Serialize,
    {
        let compiler_location = Location::caller();
        self.runtime
            .loop_on_inference_var(infer, compiler_location, &self.log, op)
    }

    /// Given a function `op` that extracts value from an inference var,
    /// returns a future that blocks until a new value is observed.
    /// A "new" value here means one not already found in `storage`;
    /// the `storage` parameter is updated to track values across invocations.
    #[track_caller]
    pub fn watch_inference_var<T>(
        &self,
        infer: InferVarIndex,
        mut op: impl FnMut(&InferenceVarData<'db>) -> T,
        storage: &mut Option<T>,
    ) -> impl Future<Output = Option<T>>
    where
        T: Serialize + Eq + Clone,
    {
        let compiler_location = Location::caller();
        self.runtime
            .loop_on_inference_var(infer, compiler_location, &self.log, move |data| {
                let new_value = op(data);

                if let Some(old_value) = &storage {
                    if *old_value == new_value {
                        return None;
                    }
                }

                *storage = Some(new_value.clone());
                Some(new_value)
            })
    }
    /// Choose between two options:
    ///
    /// * If the current node is required, then execute `if_required`. This is preferred
    ///   because it will generate stronger inference constraints.
    /// * If the current node is not required, execute `not_required` until it returns
    ///   true or false.
    #[track_caller]
    pub fn if_required(
        &mut self,
        alternative: &mut Alternative<'_>,
        mut is_required: impl AsyncFnMut(&mut Env<'db>) -> Errors<()>,
        mut not_required: impl AsyncFnMut(&mut Env<'db>) -> Errors<bool>,
    ) -> impl Future<Output = Errors<bool>> {
        let this = &*self;
        let compiler_location = Location::caller();

        let mut is_required = Box::pin(async move {
            let mut env =
                this.fork(|log| log.spawn(compiler_location, TaskDescription::IfRequired));
            is_required(&mut env).await
        });

        let mut not_required = Box::pin(async move {
            let mut env =
                this.fork(|log| log.spawn(compiler_location, TaskDescription::IfNotRequired));
            not_required(&mut env).await
        });

        std::future::poll_fn(move |cx| {
            if alternative.is_required() {
                match is_required.poll_unpin(cx) {
                    Poll::Ready(Ok(())) => Poll::Ready(Ok(true)),
                    Poll::Ready(Err(reported)) => Poll::Ready(Err(reported)),
                    Poll::Pending => Poll::Pending,
                }
            } else {
                not_required.poll_unpin(cx)
            }
        })
    }

    /// Invoke `op` for each new lower (or upper, depending on direction) bound on `?X`.
    pub async fn for_each_bound(
        &mut self,
        direction: Direction,
        infer: InferVarIndex,
        mut op: impl AsyncFnMut(&mut Env<'db>, &RedTy<'db>, ArcOrElse<'db>) -> Errors<()>,
    ) -> Errors<()> {
        let mut previous_red_ty = None;
        loop {
            let new_pair = self
                .loop_on_inference_var(infer, |data| {
                    let (red_ty, or_else) = data.red_ty_bound(direction)?;
                    if let Some(previous_ty) = &previous_red_ty {
                        if red_ty == *previous_ty {
                            return None;
                        }
                    }
                    Some((red_ty, or_else))
                })
                .await;

            match new_pair {
                None => return Ok(()),
                Some((lower_red_ty, or_else)) => {
                    self.log("for_each_bound", &[&infer, &lower_red_ty]);
                    previous_red_ty = Some(lower_red_ty);
                    op(self, previous_red_ty.as_ref().unwrap(), or_else).await?;
                }
            }
        }
    }
}

pub struct RequireAll<'env, 'db> {
    env: &'env Env<'db>,
    required: Vec<LocalBoxFuture<'env, Errors<()>>>,
}

impl<'env, 'db> RequireAll<'env, 'db> {
    #[track_caller]
    pub fn require(mut self, op: impl AsyncFnOnce(&mut Env<'db>) -> Errors<()> + 'env) -> Self {
        let index = self.required.len();
        let compiler_location = Location::caller();
        let mut env = self
            .env
            .fork(|log| log.spawn(compiler_location, TaskDescription::All(index)));
        let future = async move { op(&mut env).await };
        self.required.push(Box::pin(future));
        self
    }

    pub async fn finish(self) -> Errors<()> {
        futures::future::try_join_all(self.required).await?;
        Ok(())
    }
}
