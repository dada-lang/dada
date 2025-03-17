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
    let flc = file_line_col(file, line, column);
    match std::fs::read_to_string(file) {
        Ok(v) => {
            match v.lines().nth(line - 1) {
                Some(l) => {
                    match l.char_indices().nth(column - 1) {
                        Some((i, _)) => {
                            let prefix = &l[..i];
                            let (bold, suffix) = split_bolded_section(&l[i..]);
                            format!("<code>{prefix}<a href='{href}'>{bold}</a>{suffix}</code> ({flc})",
                                prefix = encode_safe(prefix),
                                bold = encode_safe(bold),
                                suffix = encode_safe(suffix),
                                href = file_line_col_href(file, line, column),
                            )
                        }
                        None => flc,
                    }
                    
                }
                None => flc,
            }
        },
        _ => flc,
    }
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
    let lc = match path.file_name() {
        Some(n) => {
            format!("{n}:{line}:{column}")
        }
        None => {
            format!("{path}:{line}:{column}")
        }
    };

    let href = file_line_col_href(file, line, column);
    format!("<a href='{href}'>{lc}</a>")
}

fn file_line_col_href(file: &str, line: usize, column: usize) -> String {
    format!("/source/{file}?line={line}&column={column}")
}

fn split_bolded_section(s: &str) -> (&str, &str) {
    let r = regex::Regex::new("[a-zA-Z_:]+").unwrap();
    if let Some(m) = r.find(s) {
        s.split_at(m.len())
    } else if let Some(ch) = s.chars().next() {
        s.split_at(ch.len_utf8())
    } else {
        ("", s)
    }
}
