use annotate_snippets::{Level, Renderer, Snippet};
use std::panic::PanicHookInfo;

/// The test runner overrides the panic hook temporarily.
/// When a panic occurs, info about the panic is recorded here.
#[derive(Debug)]
pub(super) struct CapturedPanic {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub message: String,
}

thread_local! {
    static LAST_PANIC: std::cell::Cell<Option<CapturedPanic>> = const { std::cell::Cell::new(None) };
}

pub(super) fn recording_panics<R>(op: impl FnOnce() -> R) -> R {
    let _guard = ReplacePanicHook::new();
    std::panic::set_hook(Box::new(|panic_hook_info| {
        let mut panic_info = CapturedPanic {
            file: "(unknown location)".to_string(),
            line: 0,
            column: 0,
            message: "(unknown panic message)".to_string(),
        };

        if let Some(message) = panic_hook_info.payload_as_str() {
            panic_info.message = message.to_string();
        }

        if let Some(location) = panic_hook_info.location() {
            panic_info.file = location.file().to_string();
            panic_info.line = location.line();
            panic_info.column = location.column();
        }

        LAST_PANIC.with(|cell| {
            cell.set(Some(panic_info));
        })
    }));
    op()
}

pub(super) fn captured_panic() -> Option<CapturedPanic> {
    LAST_PANIC.with(|cell| cell.take())
}

struct ReplacePanicHook {
    #[allow(clippy::type_complexity)]
    old_hook: Option<Box<dyn Fn(&PanicHookInfo<'_>) + Sync + Send + 'static>>,
}

impl ReplacePanicHook {
    fn new() -> Self {
        Self {
            old_hook: Some(std::panic::take_hook()),
        }
    }
}

impl Drop for ReplacePanicHook {
    fn drop(&mut self) {
        std::panic::set_hook(self.old_hook.take().unwrap());
    }
}

impl CapturedPanic {
    pub(super) fn render(&self) -> String {
        let Ok(source_contents) = std::fs::read_to_string(&self.file) else {
            return format!(
                "{}:{}:{}: {}",
                self.file, self.line, self.column, self.message
            );
        };

        let line_starts = std::iter::once(0)
            .chain(
                source_contents
                    .char_indices()
                    .filter_map(|(i, c)| (c == '\n').then_some(i + 1)),
            )
            .chain(std::iter::once(source_contents.len()))
            .collect::<Vec<_>>();

        let error_offset = line_starts[self.line as usize - 1] + self.column as usize - 1;
        let error_range = error_offset..error_offset + 1;

        Renderer::plain()
            .render(
                Level::Error.title(&self.message).snippet(
                    Snippet::source(&source_contents)
                        .line_start(1)
                        .origin(&self.file)
                        .fold(true)
                        .annotation(Level::Error.span(error_range).label(&self.message)),
                ),
            )
            .to_string()
    }
}
