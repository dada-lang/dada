use dada_ir_ast::{DebugEvent, DebugEventPayload};
use handlebars::handlebars_helper;
use rust_embed::Embed;

use crate::server::State;

pub async fn try_view(event_index: usize, state: &State) -> anyhow::Result<String> {
    let Some(event_data) = state.debug_events.lock().unwrap().get(event_index).cloned() else {
        anyhow::bail!("Event not found");
    };

    let DebugEvent { payload, .. } = &*event_data;
    match payload {
        DebugEventPayload::CheckLog(log) => Ok(crate::hbs::render("log", &log)?),
        DebugEventPayload::Diagnostic(_) => {
            anyhow::bail!("not implemented: view diagnostics")
        }
    }
}

pub async fn try_view_data(event_index: usize, state: &State) -> anyhow::Result<serde_json::Value> {
    let Some(event_data) = state.debug_events.lock().unwrap().get(event_index).cloned() else {
        anyhow::bail!("Event not found");
    };

    let DebugEvent { payload, .. } = &*event_data;
    match payload {
        DebugEventPayload::CheckLog(log) => Ok(log.clone()),
        DebugEventPayload::Diagnostic(_) => {
            anyhow::bail!("not implemented: view diagnostics")
        }
    }
}

#[derive(Embed)]
#[folder = "templates"]
struct Assets;

handlebars_helper!(index: |events: array, i: usize| events[i].clone());
