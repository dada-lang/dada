use handlebars::handlebars_helper;
use rust_embed::Embed;
use serde::Serialize;

#[derive(Embed)]
#[folder = "templates"]
struct Assets;

handlebars_helper!(index: |events: array, i: usize| events[i].clone());

pub(crate) fn render(name: &str, data: &impl Serialize) -> anyhow::Result<String> {
    let mut handlers = handlebars::Handlebars::new();
    handlers.register_embed_templates_with_extension::<Assets>(".hbs")?;
    handlers.register_helper("index", Box::new(index));
    Ok(handlers.render(name, data)?)
}

