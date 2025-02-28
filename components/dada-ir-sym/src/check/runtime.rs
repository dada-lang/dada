use std::{
    future::Future,
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    task::{Context, Poll, Waker},
};

use crate::ir::{indices::InferVarIndex, types::SymGenericKind};
use check_task::CheckTask;
use dada_ir_ast::{
    diagnostic::{Diagnostic, Err, Errors, Level},
    span::Span,
};
use dada_util::{Map, vecext::VecExt};

use crate::{check::env::Env, check::inference::InferenceVarData, check::universe::Universe};

use super::{chains::Chain, predicates::Predicate, report::OrElse};

#[derive(Clone)]
pub(crate) struct Runtime<'db> {
    data: Arc<RuntimeData<'db>>,
}

pub(crate) struct RuntimeData<'db> {
    pub db: &'db dyn crate::Db,
    inference_vars: RwLock<Vec<InferenceVarData<'db>>>,
    ready_to_execute: Mutex<Vec<Arc<CheckTask>>>,
    waiting_on_inference_var: Mutex<Map<InferVarIndex, Vec<EqWaker>>>,
    complete: AtomicBool,
    next_task_id: AtomicU64,
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

impl<'db> Runtime<'db> {
    pub(crate) fn execute<T: 'db, R: 'db>(
        db: &'db dyn crate::Db,
        span: Span<'db>,
        constrain: impl AsyncFnOnce(&Runtime<'db>) -> T + 'db,
        cleanup: impl FnOnce(T) -> R + 'db,
    ) -> R
    where
        R: Err<'db>,
    {
        let runtime = Runtime::new(db);
        let (channel_tx, channel_rx) = std::sync::mpsc::channel();
        runtime.spawn({
            let runtime = runtime.clone();
            async move {
                let result = constrain(&runtime).await;
                channel_tx.send(result).unwrap();
            }
        });
        runtime.drain();
        runtime.complete.store(true, Ordering::Relaxed);

        // Once we have reached the "complete" state, we should awaken all remaining tasks (?).

        match channel_rx.try_recv() {
            Ok(v) => cleanup(v),

            // FIXME: Obviously we need a better error message than this!
            Err(_) => R::err(db, runtime.report_type_annotations_needed(span)),
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
                next_task_id: Default::default(),
            }),
        }
    }

    fn next_task_id(&self) -> u64 {
        self.data.next_task_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Spawn a new check-task.
    fn spawn(&self, future: impl Future<Output = ()> + 'db) {
        let task = CheckTask::new(self, future);
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

    /// Returns `true` if we have fully constructed the object IR for a given function.
    /// Once this returns true, no more bounds will be added to inference variables.
    pub fn check_complete(&self) -> bool {
        self.complete.load(Ordering::Relaxed)
    }

    /// Creates a fresh inference variable of the given kind and universe.
    ///
    /// Low-level routine not to be directly invoked.
    pub fn fresh_inference_var(
        &self,
        kind: SymGenericKind,
        universe: Universe,
        span: Span<'db>,
    ) -> InferVarIndex {
        assert!(!self.check_complete());
        let mut inference_vars = self.inference_vars.write().unwrap();
        let var_index = InferVarIndex::from(inference_vars.len());
        inference_vars.push(InferenceVarData::new(kind, universe, span));
        var_index
    }

    /// Returns a future that blocks the current task until `op` returns `Some`.
    /// `op` will be reinvoked each time the state of the inference variable may have changed.
    pub fn loop_on_inference_var<T>(
        &self,
        infer: InferVarIndex,
        mut op: impl FnMut(&InferenceVarData<'db>) -> Option<T>,
    ) -> impl Future<Output = Option<T>> {
        std::future::poll_fn(move |cx| {
            let data = self.with_inference_var_data(infer, |data| op(data));
            match data {
                Some(v) => Poll::Ready(Some(v)),
                None => {
                    if self.check_complete() {
                        Poll::Ready(None)
                    } else {
                        self.block_on_inference_var(infer, cx);
                        Poll::Pending
                    }
                }
            }
        })
    }

    /// Read the current data for the given inference variable.
    ///
    /// A lock is held while the read occurs; deadlock will occur if there is an
    /// attempt to mutate inference var data during the read.
    pub fn with_inference_var_data<T>(
        &self,
        infer: InferVarIndex,
        op: impl FnOnce(&InferenceVarData<'db>) -> T,
    ) -> T {
        let inference_vars = self.inference_vars.read().unwrap();
        op(&inference_vars[infer.as_usize()])
    }

    /// See [`InferenceVarData::require_is`][]. Low-level function not to be casually invoked.
    pub fn require_inference_var_is(
        &self,
        infer: InferVarIndex,
        predicate: Predicate,
        or_else: &dyn OrElse<'db>,
    ) -> Option<Arc<dyn OrElse<'db> + 'db>> {
        self.mutate_inference_var_data_and_wake(infer, |data| data.require_is(predicate, or_else))
    }

    /// See [`InferenceVarData::require_isnt`][]. Low-level function not to be casually invoked.
    pub fn require_inference_var_isnt(
        &self,
        infer: InferVarIndex,
        predicate: Predicate,
        or_else: &dyn OrElse<'db>,
    ) -> Option<Arc<dyn OrElse<'db> + 'db>> {
        self.mutate_inference_var_data_and_wake(infer, |data| data.require_isnt(predicate, or_else))
    }

    /// See [`InferenceVarData::insert_lower_chain`][]. Low-level function not to be casually invoked.
    pub fn insert_lower_chain(
        &self,
        infer: InferVarIndex,
        chain: &Chain<'db>,
        or_else: &dyn OrElse<'db>,
    ) -> Option<Arc<dyn OrElse<'db> + 'db>> {
        self.mutate_inference_var_data_and_wake(infer, |data| {
            data.insert_lower_chain(chain, or_else)
        })
    }

    /// See [`InferenceVarData::insert_upper_chain`][]. Low-level function not to be casually invoked.
    pub fn insert_upper_chain(
        &self,
        infer: InferVarIndex,
        chain: &Chain<'db>,
        or_else: &dyn OrElse<'db>,
    ) -> Option<Arc<dyn OrElse<'db> + 'db>> {
        self.mutate_inference_var_data_and_wake(infer, |data| {
            data.insert_upper_chain(chain, or_else)
        })
    }

    fn mutate_inference_var_data_and_wake(
        &self,
        infer: InferVarIndex,
        op: impl FnOnce(&mut InferenceVarData<'db>) -> Option<Arc<dyn OrElse<'db> + 'db>>,
    ) -> Option<Arc<dyn OrElse<'db> + 'db>> {
        assert!(!self.check_complete());
        let mut inference_vars = self.inference_vars.write().unwrap();
        let inference_var = &mut inference_vars[infer.as_usize()];
        let Some(or_else) = op(inference_var) else {
            return None;
        };
        self.wake_tasks_monitoring_inference_var(infer);
        Some(or_else)
    }

    fn wake_tasks_monitoring_inference_var(&self, infer: InferVarIndex) {
        let mut waiting_on_inference_var = self.waiting_on_inference_var.lock().unwrap();
        let wakers = waiting_on_inference_var.remove(&infer);
        for EqWaker { waker } in wakers.into_iter().flatten() {
            waker.wake();
        }
    }

    /// Execute the given future asynchronously from the main execution.
    /// It must execute to completion eventually or an error will be reported.
    pub fn defer<R>(&self, env: &Env<'db>, check: impl 'db + AsyncFnOnce(Env<'db>) -> R)
    where
        R: DeferResult,
    {
        let future = check(env.clone());
        self.spawn(async move { future.await.finish() });
    }

    /// Block the current task on changes to the given inference variable.
    ///
    /// # Panics
    ///
    /// If called when [`Self::check_complete`][] returns true.
    fn block_on_inference_var(&self, infer: InferVarIndex, cx: &mut Context<'_>) {
        assert!(!self.check_complete());
        let mut waiting_on_inference_var = self.waiting_on_inference_var.lock().unwrap();
        waiting_on_inference_var
            .entry(infer)
            .or_default()
            .push_if_not_contained(EqWaker::new(cx.waker()));
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
        for (var, _) in waiting_on_inference_var.iter() {
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

    /// Execute `output` synchronously after type check constraints are gathered.
    /// Since type check constraints are gathered, we know it will never block.
    pub(crate) fn assert_check_complete<T>(&self, output: impl Future<Output = T>) -> T {
        assert!(
            self.check_complete(),
            "type inference constraints not yet complete"
        );
        futures::executor::block_on(output)
    }
}

mod check_task {
    use dada_util::log::LogState;
    use futures::{FutureExt, future::LocalBoxFuture};
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

        /// Unique identifier for this task, used for debugging.
        id: u64,

        /// Erased type: `CheckTaskState<'chk>`
        state: Mutex<CheckTaskState<'static>>,
    }

    enum CheckTaskState<'chk> {
        Executing,
        Waiting(LocalBoxFuture<'chk, ()>, LogState),
        Complete,
    }

    impl CheckTask {
        pub(super) fn new<'db>(
            runtime: &Runtime<'db>,
            future: impl Future<Output = ()> + 'db,
        ) -> Arc<Self> {
            let this = {
                let my_check = runtime.clone();

                // UNSAFE: Erase lifetimes as described on [`CheckTask`][] above.
                let my_check =
                    unsafe { std::mem::transmute::<Runtime<'db>, Runtime<'static>>(my_check) };

                Arc::new(Self {
                    runtime: my_check,
                    id: runtime.next_task_id(),
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

            let old_state = self.replace_state(CheckTaskState::Waiting(future, LogState::get()));

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

                CheckTaskState::Waiting(mut future, log_state) => {
                    let _log = dada_util::log::enter_task(self.id, log_state);
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

/// A trait to process the items that can result from a `Defer`.
pub(crate) trait DeferResult {
    fn finish(self);
}

impl DeferResult for () {
    fn finish(self) {}
}

impl<T: DeferResult> DeferResult for Errors<T> {
    fn finish(self) {
        match self {
            Ok(v) => v.finish(),
            Err(_reported) => (),
        }
    }
}
