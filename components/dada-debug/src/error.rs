pub fn error(e: anyhow::Error) -> String {
    format!("<html><body><h1>Oh geez</h1><p>{e}</p></body></html>")
}
