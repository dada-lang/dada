use std::path::{Path, PathBuf};

use eyre::Context;
use lsp_types::Diagnostic;
use regex::Regex;

mod lsp_client;

#[derive(structopt::StructOpt)]
pub struct Options {
    #[structopt(parse(from_os_str), default_value = "dada_tests")]
    dada_path: Vec<PathBuf>,

    #[structopt(long)]
    bless: bool,
}

impl Options {
    pub fn main(&self, _crate_options: &crate::Options) -> eyre::Result<()> {
        let mut total = 0;
        let mut errors = Errors::default();

        if self.dada_path.is_empty() {
            eyre::bail!("no test paths given; try --dada-path");
        }

        for root in &self.dada_path {
            for entry in walkdir::WalkDir::new(root) {
                let run_test = || -> eyre::Result<()> {
                    let entry = entry?;
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext == "dada" {
                            total += 1;
                            self.test_dada_file(path)
                                .with_context(|| format!("testing `{}`", path.display()))?;
                        }
                    }
                    Ok(())
                };

                errors.push_result(run_test());
            }
        }

        if total == 0 {
            eyre::bail!(
                "no tests found in {}",
                self.dada_path
                    .iter()
                    .map(|p| format!("`{}`", p.display()))
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        }

        let num_errors = errors.reports.len();
        for error in errors.reports {
            eprintln!("{error:?}");
        }

        eprintln!("{total} tests executed");

        if num_errors == 0 {
            Ok(())
        } else {
            eyre::bail!("{} tests failed", num_errors)
        }
    }

    fn test_dada_file(&self, path: &Path) -> eyre::Result<()> {
        eprintln!("test_data_file({})", path.display());

        let mut c = lsp_client::ChildSession::spawn();
        c.send_init()?;
        c.send_open(path)?;
        let diagnostics = c.receive_errors()?;

        let mut errors = Errors::default();

        // First, go through any expected diagnostics and make sure that
        // they match against *something* in the results.
        let expected_diagnostics = expected_diagnostics(path)?;
        for e in expected_diagnostics {
            if !diagnostics.iter().any(|d| e.matches(d)) {
                errors.push(ExpectedDiagnosticNotFound(e));
            }
        }

        // Second, compare the full details to the `.ref` file.
        // If we are in DADA_BLESS mode, then update the `.ref` file.
        let ref_path = path.with_extension("ref");
        let actual_diagnostics = format!("{:#?}", diagnostics);
        self.maybe_bless_file(&ref_path, &actual_diagnostics)?;
        let ref_contents = std::fs::read_to_string(&ref_path)
            .with_context(|| format!("reading `{}`", ref_path.display()))?;
        if ref_contents != actual_diagnostics {
            errors.push(RefOutputDoesNotMatch {
                expected: ref_contents,
                actual: actual_diagnostics,
            });
        }

        errors.into_result()
    }

    fn maybe_bless_file(&self, ref_path: &Path, actual_diagnostics: &str) -> eyre::Result<()> {
        if self.bless {
            std::fs::write(&ref_path, actual_diagnostics)
                .with_context(|| format!("writing `{}`", ref_path.display()))?;
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
struct Errors {
    reports: Vec<eyre::Report>,
}

impl Errors {
    fn push_result(&mut self, r: eyre::Result<()>) {
        if let Err(e) = r {
            self.reports.push(e);
        }
    }

    fn push(&mut self, m: impl std::error::Error + Send + Sync + 'static) {
        self.reports.push(eyre::Report::new(m));
    }

    fn into_result(mut self) -> eyre::Result<()> {
        if self.reports.is_empty() {
            return Ok(());
        }

        let r = self.reports.remove(0);
        if self.reports.is_empty() {
            return Err(r);
        }

        let others = OtherErrors::new(self.reports);
        Err(r.wrap_err(others))
    }
}

#[derive(Debug)]
struct OtherErrors {
    #[allow(dead_code)] // used just for Debug
    others: Vec<eyre::Report>,
}

impl OtherErrors {
    pub fn new(others: Vec<eyre::Report>) -> Self {
        Self { others }
    }
}

impl std::fmt::Display for OtherErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:#?}")
    }
}

#[derive(Debug)]
struct ExpectedDiagnosticNotFound(ExpectedDiagnostic);

impl std::error::Error for ExpectedDiagnosticNotFound {}

impl std::fmt::Display for ExpectedDiagnosticNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:#?}")
    }
}

#[derive(Debug)]
struct RefOutputDoesNotMatch {
    expected: String,
    actual: String,
}

impl std::error::Error for RefOutputDoesNotMatch {}

impl std::fmt::Display for RefOutputDoesNotMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            similar::TextDiff::from_lines(&self.expected, &self.actual).unified_diff()
        )
    }
}

#[derive(Debug)]
struct ExpectedDiagnostic {
    start_line: u32,
    start_column: Option<u32>,
    end_line_column: Option<(u32, u32)>,
    severity: Option<String>,
    message: Regex,
}

fn expected_diagnostics(path: &Path) -> eyre::Result<Vec<ExpectedDiagnostic>> {
    let file_contents = std::fs::read_to_string(path)?;

    let re = regex::Regex::new(r"^\s*//! ((?P<severity>[^ ]+))? (?P<msg>.*)").unwrap();

    let mut last_code_line = 1;
    let mut result = vec![];
    for (line, line_number) in file_contents.lines().zip(1..) {
        if let Some(c) = re.captures(line) {
            result.push(ExpectedDiagnostic {
                start_line: last_code_line,
                start_column: None,
                end_line_column: None,
                severity: c.name("severity").map(|s| s.as_str().to_string()),
                message: Regex::new(&c["msg"])?,
            });
        } else {
            last_code_line = line_number;
        }
    }
    Ok(result)
}
impl ExpectedDiagnostic {
    fn matches(&self, diagnostic: &Diagnostic) -> bool {
        if diagnostic.range.start.line != self.start_line {
            return false;
        }

        if let Some(start_column) = self.start_column {
            if diagnostic.range.start.character != start_column {
                return false;
            }
        }

        if let Some((end_line, end_column)) = self.end_line_column {
            if diagnostic.range.end.line != end_line {
                return false;
            }
            if diagnostic.range.end.character != end_column {
                return false;
            }
        }

        // Check the severity against the one provided (if any).
        match (&self.severity, &diagnostic.severity) {
            (Some(s), Some(d)) if s == &format!("{d:?}") => {}
            (None, None) => {}
            _ => return false,
        }

        self.message.is_match(&diagnostic.message)
    }
}
