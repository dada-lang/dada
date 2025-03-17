use serde::Serialize;

pub fn try_source(path: &str, line: u32, _column: u32) -> anyhow::Result<String> {
    const WINDOW: u32 = 10;
    let source = std::fs::read_to_string(path)?;
    let start_from = line.saturating_sub(WINDOW);
    let excerpt = source
        .lines()
        .skip(start_from as usize)
        .take((WINDOW * 2) as usize)
        .collect::<Vec<&str>>()
        .join("\n");
    crate::hbs::render(
        "source",
        &SourceArgs {
            path,
            source: excerpt,
            line,
            start_from,
        },
    )
}

#[derive(Serialize)]
struct SourceArgs<'a> {
    path: &'a str,
    source: String,
    line: u32,

    #[serde(rename = "startFrom")]
    start_from: u32,
}
