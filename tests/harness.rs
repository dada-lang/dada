fn main() -> eyre::Result<()> {
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_dada"))
        .arg("test")
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(match status.code() {
            Some(code) => eyre::format_err!("dada test exited with status code: {}", code),
            None => eyre::format_err!("dada test terminated by signal"),
        })
    }
}
