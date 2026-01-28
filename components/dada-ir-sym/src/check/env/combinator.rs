use std::{panic::Location, pin::pin};

use dada_ir_ast::diagnostic::Errors;
use futures::{
    StreamExt,
    future::{Either, LocalBoxFuture},
    stream::FuturesUnordered,
};
use serde::Serialize;

use crate::{
    check::{
        debug::TaskDescription,
        inference::Direction,
        red::RedTy,
        report::{Because, OrElse},
    },
    ir::{indices::InferVarIndex, types::SymPerm},
};

use crate::check::{env::Env, inference::InferenceVarData, report::ArcOrElse};

use super::infer_bounds::{RedPermBoundIterator, RedTyBoundIterator, SymGenericTermBoundIterator};

impl<'db> Env<'db> {
    pub async fn require(
        &mut self,
        a: impl AsyncFnOnce(&mut Env<'db>) -> Errors<bool>,
        because: impl FnOnce(&mut Env<'db>) -> Because<'db>,
        or_else: &dyn OrElse<'db>,
    ) -> Errors<()> {
        if a(self).await? {
            Ok(())
        } else {
            let because = because(self);
            Err(or_else.report(self, because))
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
            let unordered = FuturesUnordered::new();
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
            let unordered = FuturesUnordered::new();
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

    /// Returns an iterator over the bounds on an inference variable
    /// that appears under `perm`, yielding terms:
    ///
    /// * If this is a permission inference variable, the result are series of permission terms.
    ///   These are directly converted from the [`RedPerm`](crate::check::red::RedPerm) bounds you get if you call [`Self::red_perm_bounds`].
    /// * If this is a type inference variable, the result are series of type terms.
    ///   They do not include the permission inference variable.
    pub fn term_bounds(
        &self,
        perm: SymPerm<'db>,
        infer: InferVarIndex,
    ) -> SymGenericTermBoundIterator<'db> {
        SymGenericTermBoundIterator::new(self, perm, infer)
    }

    /// Returns an iterator over the red perm bounds on a permission inference variable.
    pub fn red_perm_bounds(&self, infer: InferVarIndex) -> RedPermBoundIterator<'db> {
        RedPermBoundIterator::new(self, infer)
    }

    /// Returns an iterator over the red ty bounds on a type inference variable.
    /// Note that each type inference variable has an associated permission
    /// inference variable and that this permission is not reflected in the red ty
    /// bound. Use [`Self::term_bounds`] to get back the complete inferred type.
    #[expect(dead_code)]
    pub fn red_ty_bounds(&self, infer: InferVarIndex) -> RedTyBoundIterator<'db> {
        RedTyBoundIterator::new(self, infer)
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
                    if let Some(previous_ty) = &previous_red_ty
                        && red_ty == *previous_ty
                    {
                        return None;
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
