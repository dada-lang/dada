pub fn error(e: anyhow::Error) -> String {
    format!("<html><body><h1>Oh geez</h1><p>{}</p></body></html>", e)
}

pub fn maybe_error(e: anyhow::Result<String>) -> String {
    e.unwrap_or_else(error)
}