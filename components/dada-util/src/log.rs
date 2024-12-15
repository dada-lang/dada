use std::thread::ThreadId;

use serde::ser::{SerializeSeq, SerializeStruct};

thread_local! {
    static INDENT_LEVEL: std::cell::Cell<usize> = std::cell::Cell::new(0);
}

#[derive(Copy, Clone)]
enum LogKind {
    Human,
    Json,
}

#[derive(Copy, Clone, serde::Serialize)]
enum MessageKind {
    Normal,
    Indent,
    Undent,
}

fn enabled() -> Option<LogKind> {
    lazy_static::lazy_static! {
        static ref ENABLED: Option<LogKind> = match std::env::var("DADA_DEBUG").as_deref() {
            Ok("json") => Some(LogKind::Json),
            Ok("human") => Some(LogKind::Human),
            Ok("1") => Some(LogKind::Human),
            Ok(value) => panic!("invalid value for DADA_DEBUG: expected `json` or `human`, found {}", value),
            Err(_) => None,
        };
    }
    *ENABLED
}

#[inline]
pub fn debug(message: &'static str, args: &[&dyn DebugArgument]) {
    match enabled() {
        Some(kind) => debug_cold(kind, MessageKind::Normal, message, args),
        None => (),
    }
}

#[inline]
pub fn debug_heading(message: &'static str, args: &[&dyn DebugArgument]) -> impl Sized {
    match enabled() {
        Some(kind) => {
            debug_cold(kind, MessageKind::Indent, message, args);
            Some(Undent)
        }
        None => None,
    }
}

struct Undent;

impl Drop for Undent {
    fn drop(&mut self) {
        debug_cold(LogKind::Human, MessageKind::Undent, "", &[]);
    }
}

pub trait DebugArgument: erased_serde::Serialize + std::fmt::Debug {}

impl<T: erased_serde::Serialize + std::fmt::Debug> DebugArgument for T {}

struct LogMessage<'a> {
    message_text: &'a str,
    message_kind: MessageKind,
    thread_id: ThreadId,
    args: LogArguments<'a>,
}

struct LogArguments<'a> {
    args: &'a [&'a dyn DebugArgument],
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

impl serde::Serialize for LogArguments<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_seq(Some(self.args.len()))?;
        for &arg in self.args {
            let arg: &dyn erased_serde::Serialize = arg;
            s.serialize_element(arg)?;
        }
        s.end()
    }
}

#[cold]
fn debug_cold(
    log_kind: LogKind,
    message_kind: MessageKind,
    message_text: &'static str,
    args: &[&dyn DebugArgument],
) {
    match log_kind {
        LogKind::Human => {
            use std::io::Write;
            let stderr = std::io::stderr().lock();

            let mut indent_level = INDENT_LEVEL.with(|level| level.get());
            if let MessageKind::Undent = message_kind {
                indent_level -= 1;
                INDENT_LEVEL.with(|level| level.set(indent_level));
            }

            let mut writer = std::io::BufWriter::new(stderr);
            let parts = message_text.split("##").collect::<Vec<_>>();
            let mut args_iter = args.iter();

            write!(writer, "{:width$}", "", width = indent_level * 2).unwrap();
            for part in parts {
                write!(writer, "{}", part).unwrap();
                if let Some(arg) = args_iter.next() {
                    write!(writer, "{arg:?}").unwrap();
                }
            }

            if let MessageKind::Indent = message_kind {
                INDENT_LEVEL.with(|level| level.set(indent_level + 1));
            }
        }
        LogKind::Json => {
            let message = LogMessage {
                message_text,
                message_kind,
                thread_id: std::thread::current().id(),
                args: LogArguments { args },
            };
            serde_json::to_writer_pretty(std::io::stderr(), &message).unwrap();
        }
    }
}
