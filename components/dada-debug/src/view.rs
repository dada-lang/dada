use std::path::Path;

use handlebars::handlebars_helper;
use rust_embed::Embed;

use crate::server::State;

pub async fn try_view(
    path: &Path,
    state: &State,
) -> anyhow::Result<String> {
    let json_path = state.path.join(path);   
    let json_str = std::fs::read_to_string(json_path)?;
    let json: serde_json::Value = serde_json::from_str(&json_str)?;

    let mut handlers = handlebars::Handlebars::new();
    handlers.register_embed_templates_with_extension::<Assets>(".hbs").unwrap();
    handlers.register_helper("index", Box::new(index));
    Ok(handlers.render("log", &json)?)
}

#[derive(Embed)]
#[folder = "templates"]
struct Assets;

handlebars_helper!(index: |events: array, i: usize| events[i].clone());
