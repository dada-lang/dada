use std::{pin::pin, task::Poll};

use dada_ir_ast::diagnostic::{Errors, Reported};
use futures::{
    FutureExt, StreamExt,
    future::{Either, LocalBoxFuture},
    stream::FuturesUnordered,
};

use crate::{check::alternatives::Alternative, ir::indices::InferVarIndex};

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

    pub async fn require_for_all<T>(
        &mut self,
        items: impl IntoIterator<Item = T>,
        f: impl AsyncFn(&mut Env<'db>, T) -> Errors<()>,
    ) -> Errors<()> {
        let _v: Vec<()> = futures::future::try_join_all(items.into_iter().map(|elem| async {
            let mut env = self.clone();
            f(&mut env, elem).await
        }))
        .await?;
        Ok(())
    }

    pub fn require_all(&mut self) -> RequireAll<'_, 'db> {
        RequireAll {
            env: self,
            required: vec![],
        }
    }

    pub async fn require_both(
        &mut self,
        first: impl AsyncFnOnce(&mut Self) -> Errors<()>,
        second: impl AsyncFnOnce(&mut Self) -> Errors<()>,
    ) -> Errors<()> {
        let ((), ()) = futures::future::try_join(
            async {
                let mut env = self.clone();
                first(&mut env).await
            },
            async {
                let mut env = self.clone();
                second(&mut env).await
            },
        )
        .await?;
        Ok(())
    }

    pub async fn join<A, B>(
        &mut self,
        first: impl AsyncFnOnce(&mut Self) -> A,
        second: impl AsyncFnOnce(&mut Self) -> B,
    ) -> (A, B) {
        futures::future::join(
            async {
                let mut env = self.clone();
                first(&mut env).await
            },
            async {
                let mut env = self.clone();
                second(&mut env).await
            },
        )
        .await
    }

    pub async fn either(
        &mut self,
        mut a: impl AsyncFnMut(&mut Env<'db>) -> Errors<bool>,
        mut b: impl AsyncFnMut(&mut Env<'db>) -> Errors<bool>,
    ) -> Errors<bool> {
        let a = pin!(async {
            let mut env = self.clone();
            a(&mut env).await
        });

        let b = pin!(async {
            let mut env = self.clone();
            b(&mut env).await
        });

        match futures::future::select(a, b).await {
            Either::Left((Ok(true), _)) | Either::Right((Ok(true), _)) => Ok(true),
            Either::Left((Err(reported), _)) | Either::Right((Err(reported), _)) => Err(reported),
            Either::Left((Ok(false), f)) => f.await,
            Either::Right((Ok(false), f)) => f.await,
        }
    }

    /// Returns true if any of the items satisfies the predicate.
    /// Returns false if not.
    /// Stops executing as soon as either an error or a true result is found.
    pub async fn for_all<T>(
        &mut self,
        items: impl IntoIterator<Item = T>,
        test_fn: impl AsyncFn(&mut Env<'db>, T) -> Errors<bool>,
    ) -> Errors<bool> {
        let mut unordered = FuturesUnordered::new();
        for item in items {
            unordered.push(async {
                let mut env = self.clone();
                test_fn(&mut env, item).await
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

    /// Returns true if any of the items satisfies the predicate.
    /// Returns false if not.
    /// Stops executing as soon as either an error or a true result is found.
    pub async fn exists<T>(
        &mut self,
        items: impl IntoIterator<Item = T>,
        test_fn: impl AsyncFn(&mut Env<'db>, T) -> Errors<bool>,
    ) -> Errors<bool> {
        let mut unordered = FuturesUnordered::new();
        for item in items {
            unordered.push(async {
                let mut env = self.clone();
                test_fn(&mut env, item).await
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

    /// True if both `a` and `b` are true. Stops as soon as one is found to be false.
    pub async fn both(
        &mut self,
        mut a: impl AsyncFnMut(&mut Env<'db>) -> Errors<bool>,
        mut b: impl AsyncFnMut(&mut Env<'db>) -> Errors<bool>,
    ) -> Errors<bool> {
        let a = async {
            let mut env = self.clone();
            a(&mut env).await
        };

        let b = async {
            let mut env = self.clone();
            b(&mut env).await
        };

        match futures::future::select(pin!(a), pin!(b)).await {
            Either::Left((Ok(false), _)) | Either::Right((Ok(false), _)) => Ok(false),
            Either::Left((Err(reported), _)) | Either::Right((Err(reported), _)) => Err(reported),
            Either::Left((Ok(true), f)) => f.await,
            Either::Right((Ok(true), f)) => f.await,
        }
    }

    pub async fn exists_infer_bound(
        &mut self,
        infer: InferVarIndex,
        direction: impl for<'a> Fn(&'a InferenceVarData<'db>) -> &'a [(Chain<'db>, ArcOrElse<'db>)],
        mut op: impl AsyncFnMut(&mut Env<'db>, Chain<'db>) -> Errors<bool>,
    ) -> Errors<bool> {
        let mut observed = 0;
        let mut stack = vec![];

        loop {
            self.extract_bounding_chains(infer, &mut observed, &mut stack, &direction)
                .await;

            while let Some(chain) = stack.pop() {
                match op(self, chain).await {
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
    pub async fn require_for_all_infer_bounds(
        &mut self,
        infer: InferVarIndex,
        direction: impl for<'a> Fn(&'a InferenceVarData<'db>) -> &'a [(Chain<'db>, ArcOrElse<'db>)],
        mut op: impl AsyncFnMut(&mut Env<'db>, Chain<'db>) -> Errors<()>,
    ) -> Errors<()> {
        let mut observed = 0;
        let mut stack = vec![];

        loop {
            self.extract_bounding_chains(infer, &mut observed, &mut stack, &direction)
                .await;

            while let Some(chain) = stack.pop() {
                match op(self, chain).await {
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
    pub async fn extract_bounding_chains(
        &mut self,
        infer: InferVarIndex,
        observed: &mut usize,
        stack: &mut Vec<Chain<'db>>,
        direction: impl for<'a> Fn(&'a InferenceVarData<'db>) -> &'a [(Chain<'db>, ArcOrElse<'db>)],
    ) {
        self.runtime()
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

    /// Choose between two options:
    ///
    /// * If the current node is required, then execute `if_required`. This is preferred
    ///   because it will generate stronger inference constraints.
    /// * If the current node is not required, execute `not_required` until it returns
    ///   true or false.
    pub fn if_required(
        &mut self,
        alternative: &mut Alternative<'_>,
        mut is_required: impl AsyncFnMut(&mut Env<'db>) -> Errors<()>,
        mut not_required: impl AsyncFnMut(&mut Env<'db>) -> Errors<bool>,
    ) -> impl Future<Output = Errors<bool>> {
        let this = &*self;
        let mut is_required = Box::pin(async move {
            let mut env = this.clone();
            is_required(&mut env).await
        });
        let mut not_required = Box::pin(async move {
            let mut env = this.clone();
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
    pub fn require(mut self, mut op: impl AsyncFnMut(&mut Env<'db>) -> Errors<()> + 'env) -> Self {
        let future = async move {
            let mut env = self.env.clone();
            op(&mut env).await
        };

        self.required.push(Box::pin(future));
        self
    }

    pub async fn finish(self) -> Errors<()> {
        futures::future::try_join_all(self.required).await?;
        Ok(())
    }
}
