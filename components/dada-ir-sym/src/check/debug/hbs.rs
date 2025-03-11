use handlebars::handlebars_helper;
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "templates"]
struct Assets;

handlebars_helper!(index: |events: array, i: usize| events[i].clone());

pub(crate) fn render(export: &super::export::Log) -> String {
    let mut handlers = handlebars::Handlebars::new();
    handlers.register_embed_templates_with_extension::<Assets>(".hbs").unwrap();
    handlers.register_helper("index", Box::new(index));
    handlers.render("log", export).unwrap()
}

