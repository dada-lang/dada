#![allow(clippy::arc_with_non_send_sync)] // FIXME: we may want to do this later?

use std::{
    future::Future,
    panic::Location,
    rc::Rc,
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    task::{Context, Poll, Waker},
};

use crate::ir::indices::InferVarIndex;
use check_task::CheckTask;
use dada_ir_ast::{
    diagnostic::{Diagnostic, Err, Errors, Level},
    span::Span,
};
use dada_util::{Map, Set, vecext::VecExt};
use serde::Serialize;

use crate::{check::env::Env, check::inference::InferenceVarData};

use super::{
    debug::{LogHandle, RootTaskDescription, TaskDescription},
    inference::InferenceVarDataChanged,
};

#[derive(Clone)]
pub(crate) struct Runtime<'db> {
    data: Rc<RuntimeData<'db>>,
}

pub(crate) struct RuntimeData<'db> {
    pub db: &'db dyn crate::Db,

    /// Stores the data for each inference variable created thus far.
    inference_vars: RwLock<Vec<InferenceVarData<'db>>>,

    /// Pairs `(a, b)` of inference variables where `a <: b` is required.
    /// We insert into this set when we are relating two inference variables.
    /// If it is a new relation, then we know we must propagate bounds.
    sub_inference_var_pairs: Mutex<Set<(InferVarIndex, InferVarIndex)>>,

    /// List of tasks that are ready to execute.
    ready_to_execute: Mutex<Vec<Arc<CheckTask>>>,

    /// List of tasks that are blocked, keyed by the variable they are blocked on.
    /// When the data for `InferVarIndex` changes, the tasks will be awoken.
    waiting_on_inference_var: Mutex<Map<InferVarIndex, Vec<EqWaker>>>,

    /// If true, inference state is frozen and will not change further.
    complete: AtomicBool,

    /// Integer indicating the next task id; each task gets a unique id.
    next_task_id: AtomicU64,

    /// Root log handle for this check. This handle is not used to record
    /// events, only to export the overall log. During the check, environments
    /// carry a log handle that is specific to the current task.
    /// This way when we log an event it is tied to the task that caused it.
    root_log: LogHandle<'db>,
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
    #[track_caller]
    pub(crate) fn execute<T, R>(
        db: &'db dyn crate::Db,
        span: Span<'db>,
        constrain: impl AsyncFnOnce(&Runtime<'db>) -> T + 'db,
        cleanup: impl FnOnce(T) -> R + 'db,
    ) -> R
    where
        T: 'db,
        R: 'db + Err<'db>,
    {
        let compiler_location = Location::caller();
        let runtime = Runtime::new(db, compiler_location, span);
        let (channel_tx, channel_rx) = std::sync::mpsc::channel();
        runtime.spawn_future({
            let runtime = runtime.clone();
            async move {
                let result = constrain(&runtime).await;
                channel_tx.send(result).unwrap();
            }
        });

        // Run all spawned tasks until no more progress can be made.
        runtime.drain();

        // Mark inference as done and drain again. This may generate fresh errors.
        runtime.mark_complete();
        runtime.drain();

        // Dump debug info
        runtime.root_log.dump(span);

        match channel_rx.try_recv() {
            Ok(v) => cleanup(v),

            // FIXME: Obviously we need a better error message than this!
            Err(_) => R::err(db, runtime.report_type_annotations_needed(span)),
        }
    }

    fn new(
        db: &'db dyn crate::Db,
        compiler_location: &'static Location<'static>,
        span: Span<'db>,
    ) -> Self {
        Self {
            data: Rc::new(RuntimeData {
                db,
                complete: Default::default(),
                inference_vars: Default::default(),
                sub_inference_var_pairs: Default::default(),
                ready_to_execute: Default::default(),
                waiting_on_inference_var: Default::default(),
                next_task_id: Default::default(),
                root_log: LogHandle::root(db, compiler_location, RootTaskDescription { span }),
            }),
        }
    }

    /// Get a duplicate of the root log handle.
    pub fn root_log(&self) -> LogHandle<'db> {
        self.root_log.duplicate_root_handle()
    }

    fn next_task_id(&self) -> u64 {
        self.data.next_task_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Spawn a new check-task.
    #[track_caller]
    fn spawn_future(&self, future: impl Future<Output = ()> + 'db) {
        let task = CheckTask::new(Location::caller(), self, future);
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

    /// Mark the inference process as complete and wake all tasks.
    fn mark_complete(&self) {
        self.complete.store(true, Ordering::Relaxed);
        let map = std::mem::replace(
            &mut *self.waiting_on_inference_var.lock().unwrap(),
            Default::default(),
        );
        for EqWaker { waker } in map.into_values().flatten() {
            waker.wake();
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
        log: &LogHandle,
        data: InferenceVarData<'db>,
    ) -> InferVarIndex {
        assert!(!self.check_complete());
        let mut inference_vars = self.inference_vars.write().unwrap();
        let infer = InferVarIndex::from(inference_vars.len());
        log.infer(Location::caller(), "fresh_inference_var", infer, &[&data]);
        inference_vars.push(data);
        infer
    }

    /// Returns a future that blocks the current task until `op` returns `Some`.
    /// `op` will be reinvoked each time the state of the inference variable may have changed.
    pub fn loop_on_inference_var<T>(
        &self,
        infer: InferVarIndex,
        compiler_location: &'static Location<'static>,
        log: &LogHandle<'db>,
        mut op: impl FnMut(&InferenceVarData<'db>) -> Option<T>,
    ) -> impl Future<Output = Option<T>>
    where
        T: Serialize,
    {
        std::future::poll_fn(move |cx| {
            log.infer(compiler_location, "loop_on_inference_var", infer, &[]);
            let data = self.with_inference_var_data(infer, |data| op(data));
            match data {
                Some(v) => {
                    log.infer(
                        compiler_location,
                        "loop_on_inference_var:success",
                        infer,
                        &[&v],
                    );
                    Poll::Ready(Some(v))
                }
                None => {
                    if self.check_complete() {
                        log.infer(compiler_location, "loop_on_inference_var:fail", infer, &[]);
                        Poll::Ready(None)
                    } else {
                        log.infer(compiler_location, "loop_on_inference_var:block", infer, &[]);
                        self.block_on_inference_var(compiler_location, log, infer, cx);
                        Poll::Pending
                    }
                }
            }
        })
    }

    /// If `infer` is a type variable, returns the permission variable associated with `infer`.
    /// If `infer` is a permission variable, just returns `infer`.
    pub fn perm_infer(&self, infer: InferVarIndex) -> InferVarIndex {
        self.with_inference_var_data(infer, |data| data.perm())
            .unwrap_or(infer)
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

    /// Modify the data for the inference variable `infer`.
    /// If the data actually changes (as indicated by the
    /// return value of `op` via the [`InferenceVarDataChanged`][]
    /// trait), then log the result and wake any tasks blocked on
    /// this inference variable.
    ///
    /// `op` should invoke one of the mutation methods on [`InferenceVarData`][]
    /// and nothing else. A write lock is held during the call so anything
    /// more complex risks deadlock.
    #[track_caller]
    pub fn mutate_inference_var_data<T>(
        &self,
        infer: InferVarIndex,
        log: &LogHandle,
        op: impl FnOnce(&mut InferenceVarData<'db>) -> T,
    ) -> T
    where
        T: InferenceVarDataChanged,
    {
        assert!(!self.check_complete());
        let mut inference_vars = self.inference_vars.write().unwrap();
        let inference_var = &mut inference_vars[infer.as_usize()];
        let result = op(inference_var);
        if result.did_change() {
            log.infer(
                Location::caller(),
                "mutate_inference_var_data",
                infer,
                &[&*inference_var],
            );
            self.wake_tasks_monitoring_inference_var(infer);
        }
        result
    }

    /// Record that `lower <: upper` must hold,
    /// returning true if this is the first time that this has been recorded
    /// or false if it has been recorded before.
    #[track_caller]
    pub fn insert_sub_infer_var_pair(
        &self,
        lower: InferVarIndex,
        upper: InferVarIndex,
        log: &LogHandle,
    ) -> bool {
        log.log(
            Location::caller(),
            "insert_sub_infer_var_pair",
            &[&lower, &upper],
        );
        self.sub_inference_var_pairs
            .lock()
            .unwrap()
            .insert((lower, upper))
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
    #[track_caller]
    pub fn spawn<R>(
        &self,
        env: &Env<'db>,
        task_description: TaskDescription<'db>,
        check: impl 'db + AsyncFnOnce(&mut Env<'db>) -> R,
    ) where
        R: DeferResult,
    {
        let compiler_location = Location::caller();
        let mut env = env.fork(|log| log.spawn(compiler_location, task_description));
        self.spawn_future(async move { check(&mut env).await.finish() });
    }

    /// Block the current task on changes to the given inference variable.
    ///
    /// # Panics
    ///
    /// If called when [`Self::check_complete`][] returns true.
    fn block_on_inference_var(
        &self,
        compiler_location: &'static Location<'static>,
        log: &LogHandle<'db>,
        infer: InferVarIndex,
        cx: &mut Context<'_>,
    ) {
        assert!(!self.check_complete());
        log.infer(
            compiler_location,
            "block_on_inference_var",
            infer,
            &[&infer],
        );

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
                "need to know the type here".to_string(),
            );
        }
        diag.report(db)
    }
}

mod check_task {
    use dada_util::log::LogState;
    use futures::{FutureExt, future::LocalBoxFuture};
    use std::{
        future::Future,
        panic::Location,
        rc::Rc,
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
        spawned_at: &'static Location<'static>,

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
            spawned_at: &'static Location<'static>,
            runtime: &Runtime<'db>,
            future: impl Future<Output = ()> + 'db,
        ) -> Arc<Self> {
            let this = {
                let my_check = runtime.clone();

                // UNSAFE: Erase lifetimes as described on [`CheckTask`][] above.
                let my_check =
                    unsafe { std::mem::transmute::<Runtime<'db>, Runtime<'static>>(my_check) };

                Arc::new(Self {
                    spawned_at,
                    runtime: my_check,
                    id: runtime.next_task_id(),
                    state: Mutex::new(CheckTaskState::Executing),
                })
            };

            this.set_to_wait_state(runtime, future.boxed_local());

            this
        }

        fn replace_state(&self, new_state: CheckTaskState<'static>) -> CheckTaskState<'static> {
            std::mem::replace(&mut *self.state.lock().unwrap(), new_state)
        }

        fn take_state<'db>(&self, from_check: &Runtime<'db>) -> CheckTaskState<'db> {
            assert!(std::ptr::addr_eq(
                Rc::as_ptr(&self.runtime.data),
                Rc::as_ptr(&from_check.data),
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
                Rc::as_ptr(&self.runtime.data),
                Rc::as_ptr(&from_check.data),
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

        pub(super) fn execute(self: Arc<Self>, from_check: &Runtime<'_>) {
            let state = self.take_state(from_check);
            match state {
                CheckTaskState::Complete => {
                    *self.state.lock().unwrap() = CheckTaskState::Complete;
                }

                CheckTaskState::Waiting(mut future, log_state) => {
                    let _log = dada_util::log::enter_task(self.id, self.spawned_at, log_state);
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
