use std::sync::Arc;

use dada_ir_ast::{span::AbsoluteOffset, DebugEvent};
use serde::Serialize;
use url::Url;

use crate::server::State;

/// Struct passed into the handlebars template to allow it to generate root event listing.
#[derive(Serialize)]
struct RootEvent {
    url: String,
    start: usize,
    end: usize,
    text: Option<String>,
    message: String,
    payload: RootEventPayload,
}

#[derive(Serialize)]
enum RootEventPayload {
    Diagnostic(String),
    CheckLog(usize),
}

// basic handler that responds with a static string
pub async fn root(
    state: &State,
) -> anyhow::Result<String> {
    let events = root_events(&state.debug_events.lock().unwrap())?;
    crate::hbs::render("index", &events)
}

fn root_events(
    events: &Vec<Arc<DebugEvent>>,
) -> anyhow::Result<Vec<RootEvent>> {
    let mut output = Vec::with_capacity(events.len());
    for (event, index) in events.iter().zip(0..) {
        let payload = match &**event.payload {
            DebugEvent::Diagnostic(diagnostic) => RootEventPayload::Diagnostic(diagnostic.message.clone()),
            DebugEvent::CheckLog(log) => RootEventPayload::CheckLog(log.index),
        };
        output.push(RootEvent {
            url: event.url.to_string(),
            start: event.start.as_usize(),
            end: event.end.as_usize(),
            text: extract_span(&event.url, event.start, event.end)?,
            message: event.message.clone(),
            payload,
        });
    }
    Ok(output)
}

fn extract_span(
    url: &Url,
    start: AbsoluteOffset,
    end: AbsoluteOffset,
) -> anyhow::Result<Option<String>> {
    // special case a path like `/prelude.dada`
    if let Some(path) = url.path().strip_prefix('/') {
        if !path.contains('/') {
            return Ok(None);
        }
    }
    
    // otherwise try to load the contents and create an excerpt
    let contents = std::fs::read_to_string(url.path())?;
    let start = start.as_usize();
    let end = end.as_usize();
    let text = &contents[start..end];
    let text = if text.len() > 65 {
        let first_40 = &text[..40];
        let last_20 = &text[text.len()-20..];
        format!("{} ... {}", first_40, last_20)
    } else {
        text.to_string()
    };
    Ok(Some(text))
}