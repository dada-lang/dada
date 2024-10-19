use std::{
    future::Future,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, RwLock,
    },
    task::{Context, Poll, Waker},
};

use check_task::CheckTask;
use dada_ir_ast::{
    diagnostic::{Diagnostic, Reported},
    span::Span,
};
use dada_ir_sym::{
    indices::SymInferVarIndex,
    symbol::SymGenericKind,
    ty::{SymGenericTerm, SymTy, Var},
};
use dada_util::Map;
use futures::future::LocalBoxFuture;
use typed_arena::Arena;

use crate::{
    bound::Bound,
    env::Env,
    inference::InferenceVarData,
    object_ir::{ObjectExpr, ObjectExprKind, ObjectPlaceExpr, ObjectTy, ObjectPlaceExprKind},
    universe::Universe,
};

type Deferred<'chk> = LocalBoxFuture<'chk, ()>;

#[derive(Clone)]
pub(crate) struct Check<'chk, 'db> {
    data: Arc<CheckData<'chk, 'db>>,
}

pub(crate) struct CheckData<'chk, 'db> {
    pub db: &'db dyn crate::Db,
    arenas: &'chk ExecutorArenas<'chk, 'db>,
    inference_vars: RwLock<Vec<InferenceVarData<'db>>>,
    ready_to_execute: Mutex<Vec<Arc<CheckTask>>>,
    waiting_on_inference_var: Mutex<Map<SymInferVarIndex, Vec<Waker>>>,
    complete: AtomicBool,
}

impl<'chk, 'db> std::ops::Deref for Check<'chk, 'db> {
    type Target = CheckData<'chk, 'db>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Default)]
pub struct ExecutorArenas<'chk, 'db> {
    expr_kinds: Arena<ObjectExprKind<'chk, 'db>>,
    place_expr_kinds: Arena<ObjectPlaceExprKind<'chk, 'db>>,
}

struct DeferredCheck<'chk, 'db> {
    env: Env<'db>,
    thunk: Box<dyn FnOnce(&Check<'chk, 'db>, Env<'db>) + 'chk>,
}

impl<'chk, 'db> Check<'chk, 'db> {
    pub(crate) fn execute<T: 'chk>(
        db: &'db dyn crate::Db,

        span: Span<'db>,

        // FIXME: This could be created internally, but https://github.com/rust-lang/rust/issues/131649
        // means that the resulting `impl for<'chk> async FnOnce()` signature doesn't
        // interact well with rustfmt. No big deal.
        arenas: &'chk ExecutorArenas<'chk, 'db>,

        op: impl 'chk + async FnOnce(&Check<'chk, 'db>) -> T,
    ) -> T
    where
        T: From<Reported>,
    {
        let check = Check::new(db, arenas);
        let (channel_tx, channel_rx) = std::sync::mpsc::channel();
        check.spawn({
            let check = check.clone();
            async move {
                let result = op(&check).await;
                channel_tx.send(result).unwrap();
            }
        });
        check.drain();

        match channel_rx.try_recv() {
            Ok(v) => v,

            // FIXME: Obviously we need a better error message than this!
            Err(_) => T::from(Diagnostic::error(db, span, "type annotations needed").report(db)),
        }
    }

    fn new(db: &'db dyn crate::Db, arenas: &'chk ExecutorArenas<'chk, 'db>) -> Self {
        Self {
            data: Arc::new(CheckData {
                db,
                arenas,
                complete: Default::default(),
                inference_vars: Default::default(),
                ready_to_execute: Default::default(),
                waiting_on_inference_var: Default::default(),
            }),
        }
    }

    /// Spawn a new check-task.
    fn spawn(&self, future: impl Future<Output = ()> + 'chk) {
        let task = CheckTask::new(self, future);
        self.ready_to_execute.lock().unwrap().push(task);
    }

    /// Continues running tasks until no more are left.
    fn drain(&self) {
        while let Some(ready) = self.ready_to_execute.lock().unwrap().pop() {
            ready.execute(self);
        }
    }

    /// Creates the interned `()` type.
    pub fn unit(&self) -> ObjectTy<'db> {
        ObjectTy::unit(self.db)
    }

    /// Returns `true` if this check has completed.
    pub fn is_complete(&self) -> bool {
        self.complete.load(Ordering::Relaxed)
    }

    /// Allocate an expression
    pub fn expr(
        &self,
        span: Span<'db>,
        ty: ObjectTy<'db>,
        kind: ObjectExprKind<'chk, 'db>,
    ) -> ObjectExpr<'chk, 'db> {
        let kind = self.arenas.expr_kinds.alloc(kind);
        ObjectExpr { span, ty, kind }
    }

    pub fn err_expr(&self, span: Span<'db>, reported: Reported) -> ObjectExpr<'chk, 'db> {
        self.expr(span, self.unit(), ObjectExprKind::Error(reported))
    }

    /// Allocate a place expression
    pub fn place_expr(
        &self,
        span: Span<'db>,
        ty: ObjectTy<'db>,
        kind: ObjectPlaceExprKind<'chk, 'db>,
    ) -> ObjectPlaceExpr<'chk, 'db> {
        let kind = self.arenas.place_expr_kinds.alloc(kind);
        ObjectPlaceExpr { span, ty, kind }
    }

    /// Create a series of semi-colon separated expressions.
    /// The final result type will be the type of the last expression.
    /// Returns `None` if exprs is empty.
    pub fn exprs(
        &self,
        exprs: impl IntoIterator<Item = ObjectExpr<'chk, 'db>>,
    ) -> Option<ObjectExpr<'chk, 'db>> {
        let mut lhs: Option<ObjectExpr<'_, '_>> = None;
        for rhs in exprs {
            lhs = Some(match lhs {
                None => rhs,
                Some(result) => self.expr(
                    result.span.to(rhs.span),
                    rhs.ty,
                    ObjectExprKind::Semi(result, rhs),
                ),
            });
        }

        lhs
    }

    pub fn fresh_inference_var(
        &self,
        kind: SymGenericKind,
        universe: Universe,
    ) -> SymGenericTerm<'db> {
        let mut inference_vars = self.inference_vars.write().unwrap();
        let var_index = SymInferVarIndex::from(inference_vars.len());
        inference_vars.push(InferenceVarData::new(kind, universe));
        SymGenericTerm::var(self.db, kind, Var::Infer(var_index))
    }

    pub fn with_inference_var_data<T>(
        &self,
        var: SymInferVarIndex,
        op: impl FnOnce(&InferenceVarData<'db>) -> T,
    ) -> T {
        let inference_vars = self.inference_vars.read().unwrap();
        op(&inference_vars[var.as_usize()])
    }

    pub fn push_inference_var_bound(
        &self,
        var: SymInferVarIndex,
        bound: Bound<SymGenericTerm<'db>>,
    ) {
        let mut inference_vars = self.inference_vars.write().unwrap();
        inference_vars[var.as_usize()].push_bound(bound);

        todo!() // have to notify wakers
    }

    /// Execute the given future asynchronously from the main execution.
    /// It must execute to completion eventually or an error will be reported.
    pub fn defer(
        &self,
        env: &Env<'db>,
        check: impl 'chk + async FnOnce(Check<'chk, 'db>, Env<'db>),
    ) {
        self.spawn(check(self.clone(), env.clone()));
    }

    pub fn block_on_inference_var(&self, var: SymInferVarIndex, cx: &mut Context<'_>) -> Poll<()> {
        if self.is_complete() {
            Poll::Ready(())
        } else {
            let mut waiting_on_inference_var = self.waiting_on_inference_var.lock().unwrap();
            waiting_on_inference_var
                .entry(var)
                .or_default()
                .push(cx.waker().clone());
            Poll::Pending
        }
    }
}

mod check_task {
    use futures::{future::LocalBoxFuture, FutureExt};
    use std::{
        future::Future,
        sync::{Arc, Mutex},
        task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    };

    use super::Check;

    /// # Safety notes
    ///
    /// This `Check` type is actually valid for some (existential) `'chk` and `'db`.
    ///
    /// We erase those from the type system and simply use `'static` in the field types.
    ///
    /// As a result, we cannot safely access `state` unless we can be sure that `'chk` and `'db`
    /// are still in scope.
    ///
    /// To do that, we keep a handle to `check` and then compare using `Arc::ptr_eq` to another `check` instance
    /// which we have threaded through as an ordinary parameter.
    ///
    /// If we are able to supply a `check` that has the same underlying `Arc`, and its type is valid,
    /// then we know that `self.check` has that same type, and that therefore the
    /// lifetimes in `self.state` are valid.
    pub(super) struct CheckTask {
        /// Erased type: `Check<'chk, 'db>`
        check: Check<'static, 'static>,

        /// Erased type: `CheckTaskState<'chk>`
        state: Mutex<CheckTaskState<'static>>,
    }

    enum CheckTaskState<'chk> {
        Executing,
        Waiting(LocalBoxFuture<'chk, ()>),
        Complete,
    }

    impl CheckTask {
        pub(super) fn new<'chk, 'db>(
            check: &Check<'chk, 'db>,
            future: impl Future<Output = ()> + 'chk,
        ) -> Arc<Self> {
            let this = {
                let my_check = check.clone();

                // UNSAFE: Erase lifetimes as described on [`CheckTask`][] above.
                let my_check = unsafe {
                    std::mem::transmute::<Check<'chk, 'db>, Check<'static, 'static>>(my_check)
                };

                Arc::new(Self {
                    check: my_check,
                    state: Mutex::new(CheckTaskState::Executing),
                })
            };

            this.set_to_wait_state(&check, future.boxed_local());

            this
        }

        fn take_state<'chk, 'db>(&self, from_check: &Check<'chk, 'db>) -> CheckTaskState<'chk> {
            assert!(std::ptr::addr_eq(
                Arc::as_ptr(&self.check.data),
                Arc::as_ptr(&from_check.data),
            ));

            let state =
                std::mem::replace(&mut *self.state.lock().unwrap(), CheckTaskState::Executing);

            // UNSAFE: Hide the lifetimes as described in the safety notes for [`CheckTask`][].
            unsafe { std::mem::transmute::<CheckTaskState<'static>, CheckTaskState<'chk>>(state) }
        }

        fn set_to_wait_state<'chk, 'db>(
            &self,
            from_check: &Check<'chk, 'db>,
            future: LocalBoxFuture<'chk, ()>,
        ) {
            assert!(std::ptr::addr_eq(
                Arc::as_ptr(&self.check.data),
                Arc::as_ptr(&from_check.data),
            ));

            // UNSAFE: Hide the lifetimes as described in the safety notes for [`CheckTask`][].
            let future = unsafe {
                std::mem::transmute::<LocalBoxFuture<'chk, ()>, LocalBoxFuture<'static, ()>>(future)
            };

            let old_state = std::mem::replace(
                &mut *self.state.lock().unwrap(),
                CheckTaskState::Waiting(future),
            );

            assert!(matches!(old_state, CheckTaskState::Executing));
        }

        fn waker(self: Arc<Self>) -> Waker {
            // SAFETY: We uphold the RawWakerVtable contract.
            // TODO: Document better.
            unsafe {
                Waker::from_raw(RawWaker::new(
                    Arc::into_raw(self) as *const (),
                    &CHECK_TASK_VTABLE,
                ))
            }
        }

        // Implement of the "Waker::wake" method.
        // Invoked when an inference variable we were blocked on has changed or something like that.
        // Adds this task to the list of ready-to-execute tasks.
        // Note that we *may* already have completed: that's ok, then executing us will be a no-op.
        fn wake(self: Arc<Self>) {
            // Subtle: the lifetime annotations on `check` are declared as `'static`
            // but they should be thought of as existential lifetimes.
            //
            // i.e., there is some 'chk and 'db that was associated with check
            // when this task is created. We don't actually know (locally, anyway)
            // that they are still valid -- `check` could have leaked via a ref-cycle.
            //
            // However, we do know that `check` is still
            // ALLOCATED, because we hold a strong reference to it.
            // We can add ourselves into the ready-to-execute list.
            //
            // The reader of this list will invoke `execute`, which will verify
            // that the lifetimes are still valid.

            let check = self.check.clone();
            let mut ready_to_execute = check.ready_to_execute.lock().unwrap();
            ready_to_execute.push(self);
        }

        pub(super) fn execute<'chk, 'db>(self: Arc<Self>, from_check: &Check<'chk, 'db>) {
            let state = self.take_state(from_check);
            match state {
                CheckTaskState::Complete => {
                    *self.state.lock().unwrap() = CheckTaskState::Complete;
                    return;
                }

                CheckTaskState::Waiting(mut future) => {
                    match Future::poll(
                        future.as_mut(),
                        &mut Context::from_waker(&self.clone().waker()),
                    ) {
                        Poll::Ready(()) => {
                            *self.state.lock().unwrap() = CheckTaskState::Complete;
                        }
                        Poll::Pending => {
                            self.set_to_wait_state(from_check, future);
                        }
                    }
                }

                CheckTaskState::Executing => {
                    // Our execution loop is not re-entrant, so it shouldn't be possible
                    // to hit the executing state while already executing.
                    unreachable!();
                }
            }
        }
    }

    const CHECK_TASK_VTABLE: RawWakerVTable = RawWakerVTable::new(
        |p| {
            let p: Arc<CheckTask> = unsafe { Arc::from_raw(p as *const CheckTask) };
            let q = p.clone();
            std::mem::forget(p);
            RawWaker::new(Arc::into_raw(q) as *const (), &CHECK_TASK_VTABLE)
        },
        |p| {
            let p: Arc<CheckTask> = unsafe { Arc::from_raw(p as *const CheckTask) };
            p.wake();
        },
        |p| {
            let p: Arc<CheckTask> = unsafe { Arc::from_raw(p as *const CheckTask) };
            p.clone().wake();
            std::mem::forget(p);
        },
        |p| {
            let p: Arc<CheckTask> = unsafe { Arc::from_raw(p as *const CheckTask) };
            std::mem::drop(p);
        },
    );
}

mod current_check {
    use std::ptr::NonNull;

    use super::Check;

    thread_local! {
        static CURRENT_CHECK: std::cell::Cell<Option<NonNull<()>>> = std::cell::Cell::new(None);
    }

    pub(super) fn with_check_set<T>(check: &Check<'_, '_>, op: impl FnOnce() -> T) {
        let ptr = NonNull::from(check);
        let ptr: NonNull<()> = ptr.cast();
        CURRENT_CHECK.with(|cell| {
            let _guard = RestoreCurrentCheck::new(cell.replace(Some(ptr)));
            op()
        });
    }

    pub(super) fn read_check<T>(op: impl for<'chk, 'db> FnOnce(&Check<'chk, 'db>) -> T) {
        CURRENT_CHECK.with(|cell| {
            if let Some(ptr) = cell.get() {
                let ptr: NonNull<Check<'_, '_>> = ptr.cast();

                // SAFETY: `with_check_set` ensures `CURRENT_CHECK` is a valid reference when set to `Some`
                op(unsafe { ptr.as_ref() })
            } else {
                panic!("no check in scope")
            }
        });
    }

    struct RestoreCurrentCheck {
        old_ptr: Option<NonNull<()>>,
    }

    impl RestoreCurrentCheck {
        fn new(old_ptr: Option<NonNull<()>>) -> Self {
            Self { old_ptr }
        }
    }

    impl Drop for RestoreCurrentCheck {
        fn drop(&mut self) {
            CURRENT_CHECK.with(|cell| {
                cell.set(self.old_ptr);
            });
        }
    }
}
