use std::{fs::File, thread::ThreadId};

use serde::ser::SerializeStruct;

thread_local! {
    static INDENT_LEVEL: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// A log state is a snapshot of the current indent level and other thread-sensitive state.
///
/// It is used to restore the state when a task is left.
pub struct LogState {
    indent_level: usize,
}

impl LogState {
    /// Get the current log state.
    pub fn get() -> Self {
        Self {
            indent_level: INDENT_LEVEL.with(|level| level.get()),
        }
    }

    /// Set the current log state back to what it was.
    fn set(&self) {
        INDENT_LEVEL.with(|level| level.set(self.indent_level));
    }
}

enum LogKind {
    Human(std::fs::File),
    Json(std::fs::File),
}

#[derive(Copy, Clone, serde::Serialize)]
enum MessageKind {
    Normal,
    Indent,
    Undent,
    EnterTask(u64),
    LeaveTask(u64),
}

impl std::fmt::Debug for MessageKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageKind::Normal => write!(f, "- "),
            MessageKind::Indent => write!(f, "> "),
            MessageKind::Undent => write!(f, "< "),
            MessageKind::EnterTask(id) => write!(f, "> task {id}"),
            MessageKind::LeaveTask(id) => write!(f, "< task {id}"),
        }
    }
}

/// True if debug logging is enabled for this execution.
pub fn is_enabled() -> bool {
    enabled().is_some()
}

fn enabled() -> &'static Option<LogKind> {
    lazy_static::lazy_static! {
        static ref ENABLED: Option<LogKind> = match std::env::var("DADA_DEBUG").as_deref() {
            Ok("json") => Some(LogKind::Json(File::create("dada_debug.json").unwrap())),
            Ok("human") => Some(LogKind::Human(File::create("dada_debug.txt").unwrap())),
            Ok("1") => Some(LogKind::Human(File::create("dada_debug.txt").unwrap())),
            Ok(value) => panic!("invalid value for DADA_DEBUG: expected `json` or `human`, found {}", value),
            Err(_) => None,
        };
    }
    &ENABLED
}

#[macro_export]
macro_rules! debug {
    ($message:literal $(, $args:expr)* $(,)?) => {
        $crate::log::debug($message, |op| op(&[$($crate::log::LogArgument { label: stringify!($args), value: &$args },)*]));
    };
}

#[macro_export]
macro_rules! debug_heading {
    ($message:literal $(, $args:expr)* $(,)?) => {
        let _log = $crate::log::debug_heading($message, |op| op(&[$($crate::log::LogArgument { label: stringify!($args), value: &$args },)*]));
    };
}

/// Enter a task during type-checking and other async bits of the code.
///
/// # Parameters
///
/// * `id`, the task ID.
/// * `log_state`, the log state of the task when it was suspended.
pub fn enter_task(id: u64, log_state: LogState) -> TaskUndent {
    let old_state = LogState::get();
    if let Some(kind) = enabled() { debug_cold(kind, MessageKind::EnterTask(id), "", &[]) }
    log_state.set();
    TaskUndent(id, old_state)
}

pub struct TaskUndent(u64, LogState);

impl Drop for TaskUndent {
    fn drop(&mut self) {
        if let Some(kind) = enabled() { debug_cold(kind, MessageKind::LeaveTask(self.0), "", &[]) }
        self.1.set();
    }
}

#[inline]
pub fn debug(message: &'static str, make_args: impl FnOnce(&dyn Fn(&[LogArgument<'_>]))) {
    if let Some(kind) = enabled() {
        make_args(&|args| debug_cold(kind, MessageKind::Normal, message, args));
    }
}

#[inline]
pub fn debug_heading(
    message: &'static str,
    make_args: impl FnOnce(&dyn Fn(&[LogArgument<'_>])),
) -> impl Sized {
    match enabled() {
        Some(kind) => {
            make_args(&|args| debug_cold(kind, MessageKind::Indent, message, args));
            Some(Undent)
        }
        None => None,
    }
}

struct Undent;

impl Drop for Undent {
    fn drop(&mut self) {
        if let Some(kind) = enabled() {
            debug_cold(kind, MessageKind::Undent, "", &[]);
        }
    }
}

pub trait DebugArgument: std::fmt::Debug {}

impl<T: std::fmt::Debug> DebugArgument for T {}

struct LogMessage<'a> {
    message_text: &'a str,
    message_kind: MessageKind,
    thread_id: ThreadId,
    args: &'a [LogArgument<'a>],
}

pub struct LogArgument<'a> {
    pub label: &'a str,
    pub value: &'a dyn DebugArgument,
}

impl serde::Serialize for LogMessage<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("LogMessage", 4)?;
        s.serialize_field("message_text", self.message_text)?;
        s.serialize_field("message_kind", &self.message_kind)?;
        s.serialize_field("thread_id", &format!("{:?}", self.thread_id))?;
        s.serialize_field("args", &self.args)?;
        s.end()
    }
}

impl serde::Serialize for LogArgument<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("LogArgument", 2)?;
        s.serialize_field("label", self.label)?;
        s.serialize_field("value", &format!("{:?}", self.value))?;
        s.end()
    }
}

#[cold]
fn debug_cold(
    log_kind: &LogKind,
    message_kind: MessageKind,
    message_text: &'static str,
    args: &[LogArgument<'_>],
) {
    match log_kind {
        LogKind::Human(file) => {
            use std::io::Write;
            let indent_level = INDENT_LEVEL.with(|level| level.get());

            let mut writer = std::io::BufWriter::new(file);
            write!(
                writer,
                "{:width$}{thread_id:?} {message_kind:?}",
                "",
                thread_id = std::thread::current().id(),
                width = indent_level * 2
            )
            .unwrap();
            write!(writer, "{}", message_text).unwrap();
            for arg in args {
                write!(
                    writer,
                    " {label}={value:?}",
                    label = arg.label,
                    value = arg.value
                )
                .unwrap();
            }

            writeln!(writer).unwrap();
            drop(writer);

            match message_kind {
                MessageKind::Undent => {
                    INDENT_LEVEL.with(|level| level.set(indent_level - 1));
                }
                MessageKind::Indent => {
                    INDENT_LEVEL.with(|level| level.set(indent_level + 1));
                }
                MessageKind::Normal | MessageKind::EnterTask(_) | MessageKind::LeaveTask(_) => (),
            }
        }
        LogKind::Json(file) => {
            let message = LogMessage {
                message_text,
                message_kind,
                thread_id: std::thread::current().id(),
                args,
            };
            let writer = std::io::BufWriter::new(file);
            serde_json::to_writer_pretty(writer, &message).unwrap();
        }
    }
}
