use std::{panic::Location, pin::pin, task::Poll};

use dada_ir_ast::diagnostic::{Errors, Reported};
use futures::{
    FutureExt, StreamExt,
    future::{Either, LocalBoxFuture},
    stream::FuturesUnordered,
};

use crate::{
    check::{alternatives::Alternative, debug::TaskDescription},
    ir::indices::InferVarIndex,
};

use crate::check::{env::Env, inference::InferenceVarData, red::Chain, report::ArcOrElse};

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
        mut a: impl AsyncFnMut(&mut Env<'db>) -> Errors<bool>,
        mut b: impl AsyncFnMut(&mut Env<'db>) -> Errors<bool>,
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
        let source_location = Location::caller();

        async move {
            let this = &*self;
            let test_fn = &test_fn;
            let mut unordered = FuturesUnordered::new();
            for (item, index) in items.into_iter().zip(0..) {
                unordered.push(async move {
                    let mut env = this
                        .fork(|handle| handle.spawn(source_location, TaskDescription::All(index)));
                    let result = test_fn(&mut env, item).await;
                    env.log_result(source_location, result)
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
        let source_location = Location::caller();

        async move {
            let this = &*self;
            let test_fn = &test_fn;
            let mut unordered = FuturesUnordered::new();
            for (item, index) in items.into_iter().zip(0..) {
                unordered.push(async move {
                    let mut env = this
                        .fork(|handle| handle.spawn(source_location, TaskDescription::Any(index)));
                    let result = test_fn(&mut env, item).await;
                    env.log_result(source_location, result)
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
        mut a: impl AsyncFnMut(&mut Env<'db>) -> Errors<bool>,
        mut b: impl AsyncFnMut(&mut Env<'db>) -> Errors<bool>,
    ) -> impl Future<Output = Errors<bool>> {
        let source_location = Location::caller();

        async move {
            let a = async {
                let mut env =
                    self.fork(|handle| handle.spawn(source_location, TaskDescription::All(0)));
                let result = a(&mut env).await;
                env.log_result(source_location, result)
            };

            let b = async {
                let mut env =
                    self.fork(|handle| handle.spawn(source_location, TaskDescription::All(1)));
                let result = b(&mut env).await;
                env.log_result(source_location, result)
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

    #[track_caller]
    pub fn exists_infer_bound(
        &mut self,
        infer: InferVarIndex,
        direction: impl for<'a> Fn(&'a InferenceVarData<'db>) -> &'a [(Chain<'db>, ArcOrElse<'db>)],
        mut op: impl AsyncFnMut(&mut Env<'db>, Chain<'db>) -> Errors<bool>,
    ) -> impl Future<Output = Errors<bool>> {
        let source_location = Location::caller();

        async move {
            self.indent_with_source_location(
                source_location,
                "exists_infer_bound",
                &[&infer],
                async |env| {
                    let mut observed = 0;
                    let mut stack = vec![];

                    loop {
                        env.extract_bounding_chains(infer, &mut observed, &mut stack, &direction)
                            .await;

                        while let Some(chain) = stack.pop() {
                            env.log("new bound", &[&chain]);
                            match op(env, chain).await {
                                Ok(true) => return Ok(true),
                                Ok(false) => (),
                                Err(reported) => return Err(reported),
                            }
                        }
                    }
                },
            )
            .await
        }
    }

    /// Invoke `op` on every bounding chain (either upper or lower determined by `direction`).
    /// Typically never returns as the full set of bounds on an inference variable is never known.
    /// Exception is if an `Err` occurs, it is propagated.
    #[track_caller]
    pub fn require_for_all_infer_bounds(
        &mut self,
        infer: InferVarIndex,
        direction: impl for<'a> Fn(&'a InferenceVarData<'db>) -> &'a [(Chain<'db>, ArcOrElse<'db>)],
        mut op: impl AsyncFnMut(&mut Env<'db>, Chain<'db>) -> Errors<()>,
    ) -> impl Future<Output = Errors<()>> {
        let source_location = Location::caller();

        async move {
            self.indent_with_source_location(
                source_location,
                "require_for_all_infer_bounds",
                &[&infer],
                async |env| {
                    let mut observed = 0;
                    let mut stack = vec![];

                    loop {
                        env.extract_bounding_chains(infer, &mut observed, &mut stack, &direction)
                            .await;

                        while let Some(chain) = stack.pop() {
                            env.log("new bound", &[&chain]);
                            match op(env, chain).await {
                                Ok(()) => (),
                                Err(reported) => return Err(reported),
                            }
                        }
                    }
                },
            )
            .await
        }
    }

    /// Monitor the inference variable `infer` and push new bounding chains (either upper or lower
    /// depending on `direction`) onto `stack`. The variable `observed` is used to track which
    /// chains have been observed from previous invocations; it should begin as `0` and it will be
    /// incremented during the call.
    pub async fn extract_bounding_chains(
        &mut self,
        infer: InferVarIndex,
        observed: &mut usize,
        stack: &mut Vec<Chain<'db>>,
        direction: impl for<'a> Fn(&'a InferenceVarData<'db>) -> &'a [(Chain<'db>, ArcOrElse<'db>)],
    ) {
        self.loop_on_inference_var(infer, |data| {
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

    #[track_caller]
    pub fn loop_on_inference_var<T>(
        &self,
        infer: InferVarIndex,
        op: impl FnMut(&InferenceVarData<'db>) -> Option<T>,
    ) -> impl Future<Output = Option<T>> {
        self.runtime.loop_on_inference_var(infer, &self.log, op)
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
        let source_location = Location::caller();

        let mut is_required = Box::pin(async move {
            let mut env = this.fork(|log| log.spawn(source_location, TaskDescription::IfRequired));
            is_required(&mut env).await
        });

        let mut not_required = Box::pin(async move {
            let mut env =
                this.fork(|log| log.spawn(source_location, TaskDescription::IfNotRequired));
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
}

pub struct RequireAll<'env, 'db> {
    env: &'env Env<'db>,
    required: Vec<LocalBoxFuture<'env, Errors<()>>>,
}

impl<'env, 'db> RequireAll<'env, 'db> {
    #[track_caller]
    pub fn require(mut self, op: impl AsyncFnOnce(&mut Env<'db>) -> Errors<()> + 'env) -> Self {
        let index = self.required.len();
        let source_location = Location::caller();
        let mut env = self
            .env
            .fork(|log| log.spawn(source_location, TaskDescription::All(index)));
        let future = async move { op(&mut env).await };
        self.required.push(Box::pin(future));
        self
    }

    pub async fn finish(self) -> Errors<()> {
        futures::future::try_join_all(self.required).await?;
        Ok(())
    }
}
