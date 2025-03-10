//! Prepare the debug log for export as JSON.

use serde::Serialize;

#[derive(Serialize)]
pub struct Log {
    pub events_flat: Vec<Event>,
    pub nested_event: NestedEvent,
    pub tasks: Vec<Task>,
}

#[derive(Serialize)]
pub struct Event {
    /// Kind of event.
    pub kind: &'static str,

    /// Debug formatted values.
    pub value: serde_json::Value,

    /// If this event spawns a task, this is its id.
    pub spawns: Option<TaskId>,
}

#[derive(Copy, Clone, Serialize)]
pub struct TimeStamp {
    pub index: usize,
}

#[derive(Serialize)]
pub struct Task {
    pub spawned_at: TimeStamp,
    pub description: String,
}

#[derive(Copy, Clone, Serialize)]
pub struct TaskId {
    pub index: usize,
}

#[derive(Serialize)]
pub struct NestedEvent {
    /// Index for this event in the "event by time" list
    pub timestamp: TimeStamp,

    /// "Children" events are either (a) the indented events,
    /// if this is an indent, or (b) the events from the
    /// spawned task, if this is a spawn.
    pub children: Vec<NestedEvent>,
}

#[derive(Copy, Clone, Serialize)]
pub struct InferId {
    pub index: usize,
}
