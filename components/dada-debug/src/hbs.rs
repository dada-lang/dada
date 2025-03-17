use camino::Utf8Path;
use handlebars::handlebars_helper;
use html_escape::encode_safe;
use rust_embed::Embed;
use serde::Serialize;

#[derive(Embed)]
#[folder = "templates"]
struct Assets;

handlebars_helper!(index: |events: array, i: usize| {
    events[i].clone()
});

handlebars_helper!(source_snippet: |file: str, line: usize, column: usize| {
    file_line_col(file, line, column)
});

pub(crate) fn render(name: &str, data: &impl Serialize) -> anyhow::Result<String> {
    let mut handlers = handlebars::Handlebars::new();
    handlers.register_embed_templates_with_extension::<Assets>(".hbs")?;
    handlers.register_helper("index", Box::new(index));
    handlers.register_helper("source_snippet", Box::new(source_snippet));
    Ok(handlers.render(name, data)?)
}

fn file_line_col(file: &str, line: usize, column: usize) -> String {
    let path = Utf8Path::new(file);
    let file_name = path.file_name().unwrap_or("rust");
    format!(
        "<a href='{href}'><img 
            alt='badge {file} {line} {column}'
            src='https://img.shields.io/badge/source-{file}:{line}:{column}-orange'
        /></a>",
        href = file_line_col_href(file, line, column),
        file = encode_safe(file_name).replace("-", "%2D"),
    )
}

fn file_line_col_href(file: &str, line: usize, column: usize) -> String {
    format!("/source/{file}?line={line}&column={column}")
}
