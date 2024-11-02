use std::{
    future::Future,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, RwLock,
    },
    task::{Context, Waker},
};

use check_task::CheckTask;
use dada_ir_ast::{
    diagnostic::{Diagnostic, Err, Level},
    span::Span,
};
use dada_ir_sym::{
    indices::{FromInferVar, InferVarIndex},
    symbol::SymGenericKind,
    ty::SymGenericTerm,
};
use dada_util::{vecset::VecSet, Map};
use futures::future::LocalBoxFuture;

use crate::{
    bound::Bound,
    env::Env,
    inference::InferenceVarData,
    object_ir::{ObjectGenericTerm, ObjectTy},
    universe::Universe,
};

type Deferred<'chk> = LocalBoxFuture<'chk, ()>;

#[derive(Clone)]
pub(crate) struct Runtime<'db> {
    data: Arc<RuntimeData<'db>>,
}

pub(crate) struct RuntimeData<'db> {
    pub db: &'db dyn crate::Db,
    inference_vars: RwLock<Vec<InferenceVarData<'db>>>,
    ready_to_execute: Mutex<Vec<Arc<CheckTask>>>,
    waiting_on_inference_var: Mutex<Map<InferVarIndex, VecSet<EqWaker>>>,
    complete: AtomicBool,
}

/// Wrapper around waker to compare its data/vtable fields by pointer equality.
/// This suffices to identify the waker for one of our tasks,
/// as we always use the same data/vtable pointer for a given task.
struct EqWaker {
    waker: Waker,
}

impl EqWaker {
    fn new(waker: &Waker) -> Self {
        Self {
            waker: waker.clone(),
        }
    }
}

impl std::cmp::PartialEq for EqWaker {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::addr_eq(self.waker.data(), other.waker.data())
            && std::ptr::addr_eq(self.waker.vtable(), other.waker.vtable())
    }
}

impl std::cmp::Eq for EqWaker {}

impl<'db> std::ops::Deref for Runtime<'db> {
    type Target = RuntimeData<'db>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

struct DeferredCheck<'db> {
    env: Env<'db>,
    thunk: Box<dyn FnOnce(&Runtime<'db>, Env<'db>) + 'db>,
}

impl<'db> Runtime<'db> {
    pub(crate) fn execute<T: 'db>(
        db: &'db dyn crate::Db,
        span: Span<'db>,
        op: impl async FnOnce(&Runtime<'db>) -> T + 'db,
    ) -> T
    where
        T: Err<'db>,
    {
        let check = Runtime::new(db);
        let (channel_tx, channel_rx) = std::sync::mpsc::channel();
        check.spawn(span, {
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
            Err(_) => T::err(db, check.report_type_annotations_needed(span)),
        }
    }

    fn new(db: &'db dyn crate::Db) -> Self {
        Self {
            data: Arc::new(RuntimeData {
                db,
                complete: Default::default(),
                inference_vars: Default::default(),
                ready_to_execute: Default::default(),
                waiting_on_inference_var: Default::default(),
            }),
        }
    }

    /// Spawn a new check-task.
    fn spawn(&self, span: Span<'db>, future: impl Future<Output = ()> + 'db) {
        let task = CheckTask::new(self, span, future);
        self.ready_to_execute.lock().unwrap().push(task);
    }

    /// Pop and return a task that is ready to execute (if any).
    fn pop_task(&self) -> Option<Arc<CheckTask>> {
        self.ready_to_execute.lock().unwrap().pop()
    }

    /// Continues running tasks until no more are left.
    fn drain(&self) {
        while let Some(ready) = self.pop_task() {
            ready.execute(self);
        }
    }

    /// Creates the interned `()` type.
    pub fn unit(&self) -> ObjectTy<'db> {
        ObjectTy::unit(self.db)
    }

    /// Returns `true` if we have fully constructed the object IR for a given function.
    /// Once this returns true, no more bounds will be added to inference variables.
    pub fn check_complete(&self) -> bool {
        self.complete.load(Ordering::Relaxed)
    }

    /// Creates a fresh inference variable of the given kind and universe.
    pub fn fresh_inference_var(
        &self,
        kind: SymGenericKind,
        universe: Universe,
        span: Span<'db>,
    ) -> SymGenericTerm<'db> {
        let mut inference_vars = self.inference_vars.write().unwrap();
        let var_index = InferVarIndex::from(inference_vars.len());
        inference_vars.push(InferenceVarData::new(kind, universe, span));
        SymGenericTerm::infer(self.db, kind, var_index)
    }

    /// Read the current data for the given inference variable.
    ///
    /// A lock is held while the read occurs; deadlock will occur if there is an
    /// attempt to mutate the data during the read.
    pub fn with_inference_var_data<T>(
        &self,
        infer: InferVarIndex,
        op: impl FnOnce(&InferenceVarData<'db>) -> T,
    ) -> T {
        let inference_vars = self.inference_vars.read().unwrap();
        op(&inference_vars[infer.as_usize()])
    }

    /// Modify the list of bounds for `var`, awakening any tasks that are monitoring this variable.
    /// This is a low-level function that should only be used as part of subtyping.
    pub fn push_inference_var_bound(
        &self,
        var: InferVarIndex,
        bound: Bound<ObjectGenericTerm<'db>>,
    ) {
        let mut inference_vars = self.inference_vars.write().unwrap();
        let mut waiting_on_inference_var = self.waiting_on_inference_var.lock().unwrap();
        inference_vars[var.as_usize()].push_bound(self.db, bound);
        let wakers = waiting_on_inference_var.remove(&var);
        for EqWaker { waker } in wakers.into_iter().flatten() {
            waker.wake();
        }
    }

    /// Execute the given future asynchronously from the main execution.
    /// It must execute to completion eventually or an error will be reported.
    pub fn defer(&self, env: &Env<'db>, span: Span<'db>, check: impl 'db + async FnOnce(Env<'db>)) {
        self.spawn(span, check(env.clone()));
    }

    /// Block the current task on new bounds being added to the given inference variable.
    /// Used as part of implementing the [`InferenceVarBounds`](`crate::bound::InferenceVarBounds`) stream.
    ///
    /// # Panics
    ///
    /// If called when [`Self::check_complete`][] returns true.
    pub fn block_on_inference_var(&self, var: InferVarIndex, cx: &mut Context<'_>) {
        assert!(!self.check_complete());
        let mut waiting_on_inference_var = self.waiting_on_inference_var.lock().unwrap();
        waiting_on_inference_var
            .entry(var)
            .or_default()
            .insert(EqWaker::new(cx.waker()));
    }

    fn report_type_annotations_needed(&self, span: Span<'db>) -> dada_ir_ast::diagnostic::Reported {
        let db = self.db;
        let mut diag = Diagnostic::error(db, span, "type annotations needed").label(
            db,
            Level::Error,
            span,
            "I need to know some of the types in this function",
        );
        let waiting_on_inference_var = self.waiting_on_inference_var.lock().unwrap();
        let inference_vars = self.inference_vars.read().unwrap();
        for (var, wakers) in waiting_on_inference_var.iter() {
            let var_data = &inference_vars[var.as_usize()];
            let var_span = var_data.span();
            diag = diag.label(
                db,
                Level::Note,
                var_span,
                format!("need to know the type here"),
            );
        }
        diag.report(db)
    }
}

mod check_task {
    use dada_ir_ast::span::Span;
    use futures::{future::LocalBoxFuture, FutureExt};
    use std::{
        future::Future,
        sync::{Arc, Mutex},
        task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    };

    use super::Runtime;

    /// # Safety notes
    ///
    /// This `Check` type is actually valid for some (existential) `'db`.
    /// We erase this from the type system and simply use `'static` in the field types.
    ///
    /// As a result, we cannot safely access `state` unless we can be sure that `'db`
    /// is still in scope.
    ///
    /// To do that, we keep a handle to `check` and then compare using `Arc::ptr_eq` to another `check` instance
    /// which we have threaded through as an ordinary parameter (whose type must therefore be valid).
    ///
    /// If we are able to supply a `check` that has the same underlying `Arc`, and its type is valid,
    /// then we know that `self.check` has that same type, and that therefore the
    /// lifetimes in `self.state` are valid.
    pub(super) struct CheckTask {
        /// Erased type: `Check<'db>`
        runtime: Runtime<'static>,

        /// Erased type: `Span<'db>`
        span: Span<'static>,

        /// Erased type: `CheckTaskState<'chk>`
        state: Mutex<CheckTaskState<'static>>,
    }

    enum CheckTaskState<'chk> {
        Executing,
        Waiting(LocalBoxFuture<'chk, ()>),
        Complete,
    }

    impl CheckTask {
        pub(super) fn new<'db>(
            runtime: &Runtime<'db>,
            span: Span<'db>,
            future: impl Future<Output = ()> + 'db,
        ) -> Arc<Self> {
            let this = {
                let my_check = runtime.clone();

                // UNSAFE: Erase lifetimes as described on [`CheckTask`][] above.
                let my_check =
                    unsafe { std::mem::transmute::<Runtime<'db>, Runtime<'static>>(my_check) };
                let span = unsafe { std::mem::transmute::<Span<'db>, Span<'static>>(span) };

                Arc::new(Self {
                    runtime: my_check,
                    span,
                    state: Mutex::new(CheckTaskState::Executing),
                })
            };

            this.set_to_wait_state(&runtime, future.boxed_local());

            this
        }

        fn replace_state(&self, new_state: CheckTaskState<'static>) -> CheckTaskState<'static> {
            std::mem::replace(&mut *self.state.lock().unwrap(), new_state)
        }

        fn take_state<'db>(&self, from_check: &Runtime<'db>) -> CheckTaskState<'db> {
            assert!(std::ptr::addr_eq(
                Arc::as_ptr(&self.runtime.data),
                Arc::as_ptr(&from_check.data),
            ));

            let state = self.replace_state(CheckTaskState::Executing);

            // UNSAFE: Hide the lifetimes as described in the safety notes for [`CheckTask`][].
            unsafe { std::mem::transmute::<CheckTaskState<'static>, CheckTaskState<'db>>(state) }
        }

        fn set_to_wait_state<'db>(
            &self,
            from_check: &Runtime<'db>,
            future: LocalBoxFuture<'db, ()>,
        ) {
            assert!(std::ptr::addr_eq(
                Arc::as_ptr(&self.runtime.data),
                Arc::as_ptr(&from_check.data),
            ));

            // UNSAFE: Hide the lifetimes as described in the safety notes for [`CheckTask`][].
            let future = unsafe {
                std::mem::transmute::<LocalBoxFuture<'db, ()>, LocalBoxFuture<'static, ()>>(future)
            };

            let old_state = self.replace_state(CheckTaskState::Waiting(future));

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

            let check = self.runtime.clone();
            let mut ready_to_execute = check.ready_to_execute.lock().unwrap();
            ready_to_execute.push(self);
        }

        pub(super) fn execute<'db>(self: Arc<Self>, from_check: &Runtime<'db>) {
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

    use super::Runtime;

    thread_local! {
        static CURRENT_CHECK: std::cell::Cell<Option<NonNull<()>>> = std::cell::Cell::new(None);
    }

    pub(super) fn with_check_set<T>(check: &Runtime<'_>, op: impl FnOnce() -> T) {
        let ptr = NonNull::from(check);
        let ptr: NonNull<()> = ptr.cast();
        CURRENT_CHECK.with(|cell| {
            let _guard = RestoreCurrentCheck::new(cell.replace(Some(ptr)));
            op()
        });
    }

    pub(super) fn read_check<T>(op: impl for<'db> FnOnce(&Runtime<'db>) -> T) {
        CURRENT_CHECK.with(|cell| {
            if let Some(ptr) = cell.get() {
                let ptr: NonNull<Runtime<'_>> = ptr.cast();

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
