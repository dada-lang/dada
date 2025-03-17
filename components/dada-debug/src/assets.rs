use handlebars::handlebars_helper;
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "assets"]
struct Assets;

handlebars_helper!(index: |events: array, i: usize| events[i].clone());

pub(crate) fn try_asset(path: &str) -> anyhow::Result<String> {
    let result = Assets::get(path).ok_or_else(|| anyhow::anyhow!("no asset `{path}` found"))?;
    let s = str::from_utf8(&result.data)?;
    Ok(s.to_string())
}
