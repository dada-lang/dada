//! Prepare the debug log for export as JSON.

use std::{borrow::Cow, panic::Location};

use serde::Serialize;

use crate::ir::indices::InferVarIndex;

#[derive(Serialize, Debug)]
pub struct Log<'a> {
    pub events_flat: Vec<Event<'a>>,
    pub nested_event: NestedEvent,
    pub infers: Vec<Infer>,
    pub tasks: Vec<Task>,
}

#[derive(Serialize, Debug)]
pub struct Event<'a> {
    /// Where in the Rust source...
    pub source_location: SourceLocation<'a>,

    /// Kind of event.
    pub kind: &'a str,

    /// Embedded JSON containing the value.
    pub value: Cow<'a, str>,

    /// If this event spawns a task, this is its id.
    pub spawns: Option<TaskId>,

    /// If this event describes creation/change to an inference variable, this is its id.
    pub infer: Option<InferVarIndex>,
}

#[derive(Serialize, Debug)]
pub struct SourceLocation<'a> {
    pub file: &'a str,
    pub line: u32,
    pub column: u32,
}

impl<'a> From<&'a Location<'a>> for SourceLocation<'a> {
    fn from(location: &'a Location<'a>) -> Self {
        Self {
            file: location.file(),
            line: location.line(),
            column: location.column(),
        }
    }
}

#[derive(Copy, Clone, Serialize, Debug)]
pub struct TimeStamp {
    pub index: usize,
}

#[derive(Serialize, Debug)]
pub struct Task {
    pub spawned_at: TimeStamp,
    pub description: String,
}

#[derive(Copy, Clone, Debug, Serialize)]
pub struct TaskId {
    pub index: usize,
}

#[derive(Serialize, Debug)]
pub struct NestedEvent {
    /// Index for this event in the "event by time" list
    pub timestamp: TimeStamp,

    /// "Children" events are either (a) the indented events,
    /// if this is an indent, or (b) the events from the
    /// spawned task, if this is a spawn.
    pub children: Vec<NestedEvent>,
}

#[derive(Copy, Clone, Serialize, Debug)]
pub struct InferId {
    pub index: usize,
}

/// Information about an inference variable
#[derive(Serialize, Debug)]
pub struct Infer {
    /// Location of the event that created the value of the variable
    pub created_at: TimeStamp,

    /// Location of each event that modified the value of the variable
    pub modifications: Vec<TimeStamp>,
}
