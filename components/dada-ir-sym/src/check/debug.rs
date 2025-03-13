#![expect(dead_code)]

use std::{
    panic::Location,
    sync::{mpsc::Sender, Arc, Mutex},
};

use dada_ir_ast::{diagnostic::Errors, span::Span, DebugEvent};

use crate::ir::{
    exprs::SymExpr,
    indices::InferVarIndex,
    types::{SymGenericTerm, SymPerm, SymTy},
};

use super::{
    predicates::Predicate,
    red::{Chain, Lien},
};

mod export;

pub struct LogHandle<'db> {
    log: Option<Arc<Mutex<Log<'db>>>>,
    task_index: TaskIndex,
}

impl<'db> LogHandle<'db> {
    pub fn root(
        db: &'db dyn crate::Db,
        source_location: &'static Location<'static>,
        root: RootTaskDescription<'db>,
    ) -> Self {
        if let Some(debug_tx) = db.debug_tx() {
            LogHandle {
                log: Some(Arc::new(Mutex::new(Log::new(db, source_location, root, debug_tx)))),
                task_index: TaskIndex::root(),
            }
        } else {
            LogHandle {
                log: None,
                task_index: TaskIndex::root(),
            }
        }
    }

    const DISABLED: Self = LogHandle {
        log: None,
        task_index: TaskIndex::root(),
    };

    pub fn spawn(
        &self,
        source_location: &'static Location<'static>,
        task_description: TaskDescription<'db>,
    ) -> Self {
        let Some(log) = &self.log else {
            return Self::DISABLED;
        };

        let mut locked_log = log.lock().unwrap();
        let spawned_task_index = locked_log.next_task_index();
        let event_index = locked_log.next_event_index();
        locked_log.push_task(Task {
            task_description,
            started_at: event_index,
        });
        locked_log.push_event(Event {
            task: self.task_index,
            source_location,
            kind: EventKind::Spawned(spawned_task_index),
        });
        locked_log.push_event(Event {
            task: spawned_task_index,
            source_location,
            kind: EventKind::TaskStart,
        });
        std::mem::drop(locked_log);

        LogHandle {
            log: Some(log.clone()),
            task_index: spawned_task_index,
        }
    }

    /// Duplicate this log handle. We assert that it is the root handle.
    /// This is because there is no *good* reason to duplicate any other handle;
    /// when new tasks are created you should use the `spawn` or other such methods
    /// to access them.
    pub fn duplicate_root_handle(&self) -> Self {
        assert_eq!(self.task_index, TaskIndex::root());
        Self {
            log: self.log.clone(),
            task_index: self.task_index,
        }
    }

    /// Push an "indenting" log, which causes subsequent log messages to be indented
    /// until `undent` is called.
    pub fn indent(
        &self,
        source_location: &'static Location<'static>,
        message: &'static str,
        values: &[&dyn ToEventArgument<'db>],
    ) {
        self.push_event(source_location, message, values, EventKind::Indent)
    }

    /// Remove one layer of indent
    pub fn undent(&self, source_location: &'static Location<'static>, message: &'static str) {
        self.push_event(source_location, message, &[], |m, _| EventKind::Undent(m))
    }

    /// Log a message with argument(s).
    pub fn log(
        &self,
        source_location: &'static Location<'static>,
        message: &'static str,
        values: &[&dyn ToEventArgument<'db>],
    ) {
        self.push_event(source_location, message, values, EventKind::Log)
    }

    fn push_event(
        &self,
        source_location: &'static Location<'static>,
        message: &'static str,
        values: &[&dyn ToEventArgument<'db>],
        kind: impl FnOnce(&'static str, EventArgument<'db>) -> EventKind<'db>,
    ) {
        let Some(log) = &self.log else {
            return;
        };

        let argument = if values.len() == 0 {
            EventArgument::Unit(())
        } else if values.len() == 1 {
            values[0].to_event_argument()
        } else {
            EventArgument::Many(values.iter().map(|v| v.to_event_argument()).collect())
        };

        let mut log = log.lock().unwrap();
        assert!(self.task_index.0 < log.tasks.len(), "task index {} is out of bounds", self.task_index.0);  
        log.push_event(Event {
            source_location,
            task: self.task_index,
            kind: kind(message, argument),
        });
    }

    pub fn dump(&self, span: Span<'db>) {
        let Some(log) = &self.log else {
            return;
        };

        let export = self.export();

        let log= log.lock().unwrap();
        let absolute_span = span.absolute_span(log.db);
        log.debug_tx.send(DebugEvent {
            url: absolute_span.source_file.url(log.db).clone(),
            start: absolute_span.start,
            end: absolute_span.end,
            payload: serde_json::to_value(export).unwrap(),
        }).unwrap();
    }

    fn export(&self) -> export::Log {
        let Some(log) = &self.log else {
            return export::Log {
                events_flat: vec![export::Event {
                    kind: "disabled",
                    value: serde_json::Value::Null,
                    spawns: None,
                }],
                nested_event: export::NestedEvent {
                    timestamp: export::TimeStamp { index: 0 },
                    children: vec![],
                },
                tasks: vec![],
            };
        };

        let log = log.lock().unwrap();

        // First: assemble the flat list of events, which is relatively straightforward.
        let events_flat: Vec<export::Event> = log
            .events
            .iter()
            .map(|event| export::Event {
                kind: match &event.kind {
                    EventKind::Root => "root",
                    EventKind::Spawned(..) => "spawned",
                    EventKind::TaskStart => "task_start",
                    EventKind::Indent(message, _) => message,
                    EventKind::Undent(_) => "end",
                    EventKind::Log(message, _) => message,
                },
                value: match &event.kind {
                    EventKind::Root => serde_json::Value::Null,
                    EventKind::TaskStart => serde_json::Value::Null,
                    EventKind::Spawned(_) => serde_json::Value::Null,
                    EventKind::Indent(_, event_argument) => self.export_value(event_argument),
                    EventKind::Undent(_) => serde_json::Value::Null,
                    EventKind::Log(_, event_argument) => self.export_value(event_argument),
                },
                spawns: match &event.kind {
                    EventKind::Root => None,
                    EventKind::TaskStart => None,
                    EventKind::Spawned(task_index) => Some(export::TaskId {
                        index: task_index.0,
                    }),
                    EventKind::Indent(..) => None,
                    EventKind::Undent(_) => None,
                    EventKind::Log(..) => None,
                },
            })
            .collect();

        // Next: assemble the list of events by task.
        let mut events_by_task: Vec<Vec<usize>> = (0..log.tasks.len()).map(|_| vec![]).collect();
        for (event, index) in log.events.iter().zip(0..) {
            events_by_task[event.task.0].push(index);
        }

        // Next: assemble the nested events.
        let root_task = TaskIndex::root();
        let nested_event = log.export_nested_event_for_task(root_task, &events_by_task);

        // Next: assemble tasks
        let tasks = vec![ /* TODO */];

        export::Log {
            events_flat,
            nested_event,
            tasks,
        }
    }

    fn export_value(&self, event_argument: &EventArgument<'db>) -> serde_json::Value {
        match event_argument {
            EventArgument::Many(event_arguments) => event_arguments
                .iter()
                .map(|a| self.export_value(a))
                .collect(),
            EventArgument::Unit(v) => serde_json::to_value(v).unwrap(),
            EventArgument::Usize(v) => serde_json::to_value(v).unwrap(),
            EventArgument::Bool(v) => serde_json::to_value(v).unwrap(),
            EventArgument::Lien(v) => serde_json::to_value(format!("{v:?}")).unwrap(),
            EventArgument::SymExpr(v) => serde_json::to_value(format!("{v:?}")).unwrap(),
            EventArgument::OptSymExpr(v) => serde_json::to_value(format!("{v:?}")).unwrap(),
            EventArgument::SymTerm(v) => serde_json::to_value(format!("{v:?}")).unwrap(),
            EventArgument::SymTy(v) => serde_json::to_value(format!("{v:?}")).unwrap(),
            EventArgument::SymPerm(v) => serde_json::to_value(format!("{v:?}")).unwrap(),
            EventArgument::InferVarIndex(v) => serde_json::to_value(export::InferId {
                index: v.as_usize(),
            })
            .unwrap(),
            EventArgument::Errors(v) => serde_json::to_value(format!("{v:?}")).unwrap(),
            EventArgument::Trivalue(v) => serde_json::to_value(format!("{v:?}")).unwrap(),
            EventArgument::Chain(v) => serde_json::to_value(format!("{v:?}")).unwrap(),
        }
    }
}

pub struct Log<'db> {
    db: &'db dyn crate::Db,
    tasks: Vec<Task<'db>>,
    events: Vec<Event<'db>>,
    inference_variables: Vec<InferenceVariable<'db>>,
    debug_tx: Sender<DebugEvent>,
}

impl<'db> Log<'db> {
    fn new(
        db: &'db dyn crate::Db,
        source_location: &'static Location<'static>,
        root: RootTaskDescription<'db>,
        debug_tx: Sender<DebugEvent>,
    ) -> Self {
        let tasks = vec![Task {
            task_description: TaskDescription::Root(root),
            started_at: EventIndex(0),
        }];

        let events = vec![Event {
            task: TaskIndex::root(),
            source_location,
            kind: EventKind::Root,
        }];

        Self {
            db,
            tasks,
            events,
            inference_variables: Default::default(),
            debug_tx,
        }
    }

    fn next_task_index(&self) -> TaskIndex {
        TaskIndex(self.tasks.len())
    }

    fn next_event_index(&self) -> EventIndex {
        EventIndex(self.events.len())
    }

    fn push_task(&mut self, task: Task<'db>) {
        self.tasks.push(task);
    }

    fn push_event(&mut self, event: Event<'db>) {
        self.events.push(event);
    }

    fn export_nested_event_for_task(
        &self,
        task: TaskIndex,
        events_by_task: &[Vec<usize>],
    ) -> export::NestedEvent {
        let Some((event_first, mut events_rest)) = events_by_task[task.0].split_first() else {
            panic!("no root event")
        };

        export::NestedEvent {
            timestamp: export::TimeStamp {
                index: *event_first,
            },
            children: self.export_child_nested_events(task, &mut events_rest, events_by_task),
        }
    }

    fn export_child_nested_events(
        &self,
        task: TaskIndex,
        task_events: &mut &[usize],
        events_by_task: &[Vec<usize>],
    ) -> Vec<export::NestedEvent> {
        let mut output = vec![];

        loop {
            let Some((event_first, events_rest)) = task_events.split_first() else {
                return output;
            };
            *task_events = events_rest;
            let event_kind = &self.events[*event_first];
            match &event_kind.kind {
                EventKind::Undent(_) => {
                    return output;
                }
                EventKind::Spawned(spawned_task) => {
                    output.push(export::NestedEvent {
                        timestamp: export::TimeStamp {
                            index: *event_first,
                        },
                        children: vec![
                            self.export_nested_event_for_task(*spawned_task, events_by_task),
                        ],
                    });
                }
                EventKind::Indent(..) => {
                    output.push(export::NestedEvent {
                        timestamp: export::TimeStamp {
                            index: *event_first,
                        },
                        children: self.export_child_nested_events(
                            task,
                            task_events,
                            events_by_task,
                        ),
                    });
                }
                EventKind::Root | EventKind::Log(..) | EventKind::TaskStart => {
                    output.push(export::NestedEvent {
                        timestamp: export::TimeStamp {
                            index: *event_first,
                        },
                        children: Default::default(),
                    });
                }
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct TaskIndex(usize);

impl TaskIndex {
    pub const fn root() -> Self {
        TaskIndex(0)
    }
}

pub struct Task<'db> {
    pub task_description: TaskDescription<'db>,
    pub started_at: EventIndex,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct EventIndex(usize);

pub struct Event<'db> {
    pub task: TaskIndex,
    pub source_location: &'static Location<'static>,
    pub kind: EventKind<'db>,
}

pub enum EventKind<'db> {
    Root,
    TaskStart,
    Spawned(TaskIndex),
    Indent(&'static str, EventArgument<'db>),
    Undent(&'static str),
    Log(&'static str, EventArgument<'db>),
}

pub struct RootTaskDescription<'db> {
    pub span: Span<'db>,
}

pub enum TaskDescription<'db> {
    Root(RootTaskDescription<'db>),
    Require(usize),
    Join(usize),
    All(usize),
    Any(usize),
    IfRequired,
    IfNotRequired,
    RequireAssignableType(SymTy<'db>, SymTy<'db>),
    RequireEqualTypes(SymTy<'db>, SymTy<'db>),
    RequireNumericType(SymTy<'db>),
    RequireFutureType(SymTy<'db>),
    RequireBoundsProvablyPredicate(InferVarIndex, Predicate),
    RequireBoundsNotProvablyPredicate(InferVarIndex, Predicate),
    RequireLowerChain,
    IfNotNever,
    Misc,
    CheckArg(usize),
}

pub trait ToEventArgument<'db> {
    fn to_event_argument(&self) -> EventArgument<'db>;
}

impl<'db, T: ?Sized + ToEventArgument<'db>> ToEventArgument<'db> for &T {
    fn to_event_argument(&self) -> EventArgument<'db> {
        T::to_event_argument(self)
    }
}

macro_rules! to_event_argument_impls {
    (
        $(#[$attr:meta])*
        $v:vis enum $EventArgument:ident<$db:lifetime> {
            $($variant:ident($ty:ty),)*
        }
    ) => {
        $(#[$attr])*
        $v enum $EventArgument<$db> {
            $($variant($ty),)*
        }

        $(
            impl<$db> ToEventArgument<$db> for $ty {
                fn to_event_argument(&self) -> $EventArgument<$db> {
                    $EventArgument::$variant(
                        <$ty>::clone(self)
                    )
                }
            }
        )*
    };
}

to_event_argument_impls! {
    #[derive(Debug, Clone)]
    pub enum EventArgument<'db> {
        Many(Vec<EventArgument<'db>>),
        Unit(()),
        Usize(usize),
        Bool(bool),
        Lien(Lien<'db>),
        SymExpr(SymExpr<'db>),
        OptSymExpr(Option<SymExpr<'db>>),
        SymTerm(SymGenericTerm<'db>),
        SymTy(SymTy<'db>),
        SymPerm(SymPerm<'db>),
        InferVarIndex(InferVarIndex),
        Errors(Errors<()>),
        Trivalue(Errors<bool>),
        Chain(Chain<'db>),
    }
}

impl<'db, T> ToEventArgument<'db> for [T]
where
    T: ToEventArgument<'db>,
{
    fn to_event_argument(&self) -> EventArgument<'db> {
        EventArgument::Many(self.iter().map(|v| v.to_event_argument()).collect())
    }
}

pub struct InferenceVariable<'db> {
    span: Span<'db>,
}
