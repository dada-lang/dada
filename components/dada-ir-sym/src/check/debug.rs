#![expect(dead_code)]

use std::{
    collections::{BTreeMap, btree_map::Entry},
    panic::Location,
    rc::Rc,
    sync::{Mutex, mpsc::Sender},
};

use dada_ir_ast::{DebugEvent, DebugEventPayload, span::Span};
use dada_util::fixed_depth_json;
use export::{SourceLocation, TimeStamp};
use serde::Serialize;

use crate::ir::{indices::InferVarIndex, types::SymTy};

use super::predicates::Predicate;

mod export;

pub struct LogHandle<'db> {
    log: Option<Rc<Mutex<Log<'db>>>>,
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
                log: Some(Rc::new(Mutex::new(Log::new(
                    db,
                    source_location,
                    root,
                    debug_tx,
                )))),
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
        values: &[&dyn erased_serde::Serialize],
    ) {
        self.push_event(source_location, message, values, |message, json_value| {
            EventKind::Indent {
                message,
                json_value,
            }
        })
    }

    /// Remove one layer of indent
    pub fn undent(&self, source_location: &'static Location<'static>, message: &'static str) {
        self.push_event(source_location, message, &[], |message, _| {
            EventKind::Undent { message }
        })
    }

    /// Log a message with argument(s).
    pub fn log(
        &self,
        source_location: &'static Location<'static>,
        message: &'static str,
        values: &[&dyn erased_serde::Serialize],
    ) {
        self.push_event(source_location, message, values, |message, json_value| {
            EventKind::Log {
                message,
                json_value,
            }
        })
    }

    /// Log a message with argument(s).
    pub fn infer(
        &self,
        source_location: &'static Location<'static>,
        message: &'static str,
        infer: InferVarIndex,
        values: &[&dyn erased_serde::Serialize],
    ) {
        self.push_event(source_location, message, values, |message, json_value| {
            EventKind::Infer {
                infer,
                message,
                json_value,
            }
        })
    }

    fn push_event(
        &self,
        source_location: &'static Location<'static>,
        message: &'static str,
        values: &[&dyn erased_serde::Serialize],
        kind: impl FnOnce(&'static str, String) -> EventKind,
    ) {
        let Some(log) = &self.log else {
            return;
        };

        let mut log = log.lock().unwrap();
        assert!(
            self.task_index.0 < log.tasks.len(),
            "task index {} is out of bounds",
            self.task_index.0
        );

        let argument = event_argument(values);

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

        let log = log.lock().unwrap();
        log.dump(span);
    }
}

pub struct Log<'db> {
    db: &'db dyn crate::Db,
    tasks: Vec<Task<'db>>,
    events: Vec<Event>,
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

    fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    fn dump(&self, span: Span<'db>) {
        let export = self.export();
        let absolute_span = span.absolute_span(self.db);

        self.debug_tx
            .send(DebugEvent {
                url: absolute_span.source_file.url(self.db).clone(),
                start: absolute_span.start,
                end: absolute_span.end,
                payload: DebugEventPayload::CheckLog(serde_json::to_value(export).unwrap()),
            })
            .unwrap();
    }

    fn export(&self) -> export::Log<'_> {
        // First: assemble the flat list of events, which is relatively straightforward.
        let events_flat: Vec<export::Event<'_>> = self
            .events
            .iter()
            .map(|event| export::Event {
                source_location: SourceLocation::from(event.source_location),
                kind: match &event.kind {
                    EventKind::Root => "root",
                    EventKind::Spawned(..) => "spawned",
                    EventKind::TaskStart => "task_start",
                    EventKind::Indent { message, .. } => message,
                    EventKind::Undent { .. } => "end",
                    EventKind::Log { message, .. } => message,
                    EventKind::Infer { message, .. } => message,
                },
                value: match &event.kind {
                    EventKind::Root => "null".into(),
                    EventKind::TaskStart => "null".into(),
                    EventKind::Spawned(task_index) => {
                        event_argument(&[&self.tasks[task_index.0].task_description]).into()
                    }
                    EventKind::Indent {
                        message: _,
                        json_value,
                    } => json_value.into(),
                    EventKind::Undent { message: _ } => "null".into(),
                    EventKind::Log {
                        message: _,
                        json_value,
                    } => json_value.into(),
                    EventKind::Infer { json_value, .. } => json_value.into(),
                },
                spawns: match &event.kind {
                    EventKind::Root => None,
                    EventKind::TaskStart => None,
                    EventKind::Spawned(task_index) => Some(export::TaskId {
                        index: task_index.0,
                    }),
                    EventKind::Indent { .. } => None,
                    EventKind::Undent { .. } => None,
                    EventKind::Log { .. } => None,
                    EventKind::Infer { .. } => None,
                },
                infer: match &event.kind {
                    EventKind::Root
                    | EventKind::TaskStart
                    | EventKind::Spawned(..)
                    | EventKind::Indent { .. }
                    | EventKind::Undent { .. }
                    | EventKind::Log { .. } => None,
                    EventKind::Infer { infer, .. } => Some(*infer),
                },
            })
            .collect();

        // Next: assemble the list of events by task.
        let mut events_by_task: Vec<Vec<usize>> = (0..self.tasks.len()).map(|_| vec![]).collect();
        for (event, index) in self.events.iter().zip(0..) {
            events_by_task[event.task.0].push(index);
        }

        // Next: assemble the nested events.
        let root_task = TaskIndex::root();
        let nested_event = self.export_nested_event_for_task(root_task, &events_by_task);

        // Assemble inference events
        let infers = self.export_infers();

        // Assemble tasks
        let tasks = self
            .tasks
            .iter()
            .map(|task| export::Task {
                spawned_at: export::TimeStamp {
                    index: task.started_at.0,
                },
                description: event_argument(&[&task.task_description]),
            })
            .collect();

        export::Log {
            events_flat,
            nested_event,
            tasks,
            infers,
        }
    }

    fn export_infers(&self) -> Vec<export::Infer> {
        let mut events_by_infer_var: BTreeMap<InferVarIndex, export::Infer> = Default::default();

        for (event, index) in self.events.iter().zip(0..) {
            if let EventKind::Infer { infer, .. } = &event.kind {
                match events_by_infer_var.entry(*infer) {
                    Entry::Vacant(e) => {
                        e.insert(export::Infer {
                            created_at: TimeStamp { index },
                            modifications: vec![],
                        });
                    }
                    Entry::Occupied(mut e) => {
                        e.get_mut().modifications.push(TimeStamp { index });
                    }
                }
            }
        }

        events_by_infer_var.into_values().collect()
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
            children: self.export_child_nested_events(&mut events_rest, events_by_task),
        }
    }

    fn export_child_nested_events(
        &self,
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
                EventKind::Undent { .. } => {
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
                EventKind::Indent { .. } => {
                    output.push(export::NestedEvent {
                        timestamp: export::TimeStamp {
                            index: *event_first,
                        },
                        children: self.export_child_nested_events(task_events, events_by_task),
                    });
                }
                EventKind::Infer { .. }
                | EventKind::Root
                | EventKind::Log { .. }
                | EventKind::TaskStart => {
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

fn event_argument(values: &[&dyn erased_serde::Serialize]) -> String {
    // FIXME: rewrite `fixed_depth_json` to not create a value

    let value = if values.is_empty() {
        serde_json::Value::Null
    } else if values.len() == 1 {
        fixed_depth_json::to_json_value_max_depth(values[0], 5)
    } else {
        fixed_depth_json::to_json_value_max_depth(&values, 5)
    };

    serde_json::to_string(&value).unwrap()
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

pub struct Event {
    pub task: TaskIndex,
    pub source_location: &'static Location<'static>,
    pub kind: EventKind,
}

pub enum EventKind {
    /// Root event of type checking
    Root,

    /// Start event for a task spawned during type checking
    TaskStart,

    /// Current task spawned the child with the given index
    Spawned(TaskIndex),

    /// Display hint: indent further logs until `Undent` encountered
    Indent {
        message: &'static str,
        json_value: String,
    },

    /// End indenting
    Undent { message: &'static str },

    /// Add a log item with the given header + (JSON-encoded) argument
    Log {
        message: &'static str,
        json_value: String,
    },

    /// A log message about an inference variable being created or modified
    Infer {
        message: &'static str,
        infer: InferVarIndex,
        json_value: String,
    },
}

#[derive(Serialize)]
pub struct RootTaskDescription<'db> {
    pub span: Span<'db>,
}

#[derive(Serialize)]
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

pub struct InferenceVariable<'db> {
    span: Span<'db>,
}
