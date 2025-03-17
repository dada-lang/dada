fn main() -> dada_util::Fallible<()> {
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_dada"))
        .arg("test")
        .arg("--")
        .arg("tests")
        .status()?;
    if status.success() {
        Ok(())
    } else {
        match status.code() {
            Some(code) => dada_util::bail!("dada test exited with status code: {}", code),
            None => dada_util::bail!("dada test terminated by signal"),
        }
    }
}
