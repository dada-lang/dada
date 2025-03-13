
use handlebars::handlebars_helper;
use rust_embed::Embed;

use crate::server::State;

pub async fn try_view(
    event_index: usize,
    state: &State,
) -> anyhow::Result<String> {
    let event_data = state.debug_events.lock().unwrap().get(event_index).cloned();
    match event_data {
        Some(d) => {
            Ok(crate::hbs::render("log", &d.payload)?)
        }
        None => {
            Err(anyhow::anyhow!("Event not found"))
        }
    }
}

#[derive(Embed)]
#[folder = "templates"]
struct Assets;

handlebars_helper!(index: |events: array, i: usize| events[i].clone());
