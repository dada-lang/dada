use serde::Serialize;

pub fn try_source(path: &str, line: u32, _column: u32) -> anyhow::Result<String> {
    let source = std::fs::read_to_string(path)?;
    crate::hbs::render("source", &SourceArgs { path, source, line })
}

#[derive(Serialize)]
struct SourceArgs<'a> {
    path: &'a str,
    source: String,
    line: u32,
}
