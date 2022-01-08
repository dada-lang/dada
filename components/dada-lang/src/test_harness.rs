use std::env;
use std::path::{Path, PathBuf};

use dada_execute::kernel::BufferKernel;
use dada_ir::{filename::Filename, item::Item};
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
    pub async fn main(&self, _crate_options: &crate::Options) -> eyre::Result<()> {
        let mut total = 0;
        let mut errors = Errors::default();

        if self.dada_path.is_empty() {
            eyre::bail!("no test paths given; try --dada-path");
        }

        const REF_EXTENSIONS: &[&str] = &["ref", "lsp", "bir", "validated", "syntax", "stdout"];

        for root in &self.dada_path {
            for entry in walkdir::WalkDir::new(root) {
                let run_test = async {
                    let entry = entry?;
                    let path = entry.path();

                    if path.is_dir() {
                        return Ok(());
                    }

                    if let Some(ext) = path.extension() {
                        if ext == "dada" {
                            total += 1;
                            self.test_dada_file(path)
                                .await
                                .with_context(|| format!("testing `{}`", path.display()))?;
                            tracing::info!("test `{}` passed", path.display());
                            return Ok(());
                        } else if REF_EXTENSIONS.iter().any(|e| *e == ext) {
                            // ignore ref files
                            return Ok(());
                        }
                    }

                    // Error out for random files -- I've frequently accidentally made
                    // tests with the extension `dad`, for example.
                    //
                    // FIXME: we should probably consider gitignore here
                    eyre::bail!("file `{}` has unrecognized extension", path.display())
                };

                errors.push_result(run_test.await);
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
            tracing::error!("{error:?}");
        }

        tracing::info!("{total} tests executed");

        if num_errors == 0 {
            Ok(())
        } else {
            eyre::bail!("{} tests failed", num_errors)
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn test_dada_file(&self, path: &Path) -> eyre::Result<()> {
        let expected_diagnostics = &expected_diagnostics(path)?;
        self.test_dada_file_normal(path, expected_diagnostics)
            .await?;
        self.test_dada_file_in_ide(path, expected_diagnostics)?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn test_dada_file_normal(
        &self,
        path: &Path,
        expected_diagnostics: &[ExpectedDiagnostic],
    ) -> eyre::Result<()> {
        let mut db = dada_db::Db::default();
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("reading `{}`", path.display()))?;
        let filename = dada_ir::filename::Filename::from(&db, path);
        db.update_file(filename, contents);
        let diagnostics = db.diagnostics(filename);

        let mut errors = Errors::default();
        self.match_diagnostics_against_expectations(
            &db,
            &diagnostics,
            expected_diagnostics,
            &mut errors,
        )?;
        self.check_output_against_ref_file(
            dada_error_format::format_diagnostics(&db, &diagnostics)?,
            &path.with_extension("ref"),
            &mut errors,
        )?;
        self.check_compiled(
            &db,
            &[filename],
            |item| db.debug_syntax_tree(item),
            &path.with_extension("syntax"),
            &mut errors,
        )?;
        self.check_compiled(
            &db,
            &[filename],
            |item| db.debug_validated_tree(item),
            &path.with_extension("validated"),
            &mut errors,
        )?;
        self.check_compiled(
            &db,
            &[filename],
            |item| db.debug_bir(item),
            &path.with_extension("bir"),
            &mut errors,
        )?;
        self.check_interpreted(&db, filename, &path.with_extension("stdout"), &mut errors)
            .await?;
        errors.into_result()
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn test_dada_file_in_ide(
        &self,
        path: &Path,
        expected_diagnostics: &[ExpectedDiagnostic],
    ) -> eyre::Result<()> {
        let mut c = lsp_client::ChildSession::spawn();
        c.send_init()?;
        c.send_open(path)?;
        let diagnostics = c.receive_errors()?;

        let mut errors = Errors::default();
        self.match_diagnostics_against_expectations(
            &(),
            &diagnostics,
            expected_diagnostics,
            &mut errors,
        )?;
        self.check_output_against_ref_file(
            format!("{:#?}", diagnostics),
            &path.with_extension("lsp"),
            &mut errors,
        )?;
        errors.into_result()
    }

    fn check_output_against_ref_file(
        &self,
        actual_output: String,
        ref_path: &Path,
        errors: &mut Errors,
    ) -> eyre::Result<()> {
        let sanitized_output = sanitize_output(actual_output)?;
        self.maybe_bless_file(ref_path, &sanitized_output)?;
        let ref_contents = std::fs::read_to_string(&ref_path)
            .with_context(|| format!("reading `{}`", ref_path.display()))?;
        if ref_contents != sanitized_output {
            errors.push(RefOutputDoesNotMatch {
                ref_path: ref_path.to_owned(),
                expected: ref_contents,
                actual: sanitized_output,
            });
        }
        Ok(())
    }

    fn maybe_bless_file(&self, ref_path: &Path, actual_diagnostics: &str) -> eyre::Result<()> {
        if self.bless {
            std::fs::write(&ref_path, actual_diagnostics)
                .with_context(|| format!("writing `{}`", ref_path.display()))?;
        }

        Ok(())
    }

    fn match_diagnostics_against_expectations<D>(
        &self,
        db: &D::Db,
        actual_diagnostics: &[D],
        expected_diagnostics: &[ExpectedDiagnostic],
        errors: &mut Errors,
    ) -> eyre::Result<()>
    where
        D: ActualDiagnostic,
    {
        let mut actual_diagnostics: Vec<D> = actual_diagnostics.to_vec();
        actual_diagnostics.sort_by_key(|a| a.start(db));

        let mut expected_iter = expected_diagnostics.iter().fuse().peekable();
        let mut actual_iter = actual_diagnostics.iter().fuse().peekable();

        let mut expected_output = Vec::new();
        let mut actual_output = Vec::new();

        while expected_iter.peek().is_some() && actual_iter.peek().is_some() {
            let actual_diagnostic = actual_iter.peek().unwrap();
            let expected_diagnostic = expected_iter.peek().unwrap();
            if actual_diagnostic.matches(db, expected_diagnostic) {
                actual_output.push(actual_diagnostic.summary(db));
                expected_output.push(actual_diagnostic.summary(db));
                actual_iter.next();
                expected_iter.next();
                continue;
            }

            let (actual_line, _) = actual_diagnostic.start(db);
            let expected_line = expected_diagnostic.start_line;

            if actual_line < expected_line {
                actual_output.push(actual_diagnostic.summary(db));
                actual_iter.next();
                continue;
            }

            expected_output.push(expected_diagnostic.summary());
            expected_iter.next();
            continue;
        }

        for actual_diagnostic in actual_iter {
            actual_output.push(actual_diagnostic.summary(db));
        }

        for expected_diagnostic in expected_iter {
            expected_output.push(expected_diagnostic.summary());
        }

        if expected_output != actual_output {
            errors.push(DiagnosticsDoNotMatch {
                expected: expected_output,
                actual: actual_output,
            });
        }

        Ok(())
    }

    fn check_compiled<D>(
        &self,
        db: &dada_db::Db,
        filenames: &[Filename],
        mut item_op: impl FnMut(Item) -> Option<D>,
        bir_path: &Path,
        errors: &mut Errors,
    ) -> eyre::Result<()>
    where
        D: std::fmt::Debug,
    {
        let items: Vec<Item> = filenames
            .iter()
            .flat_map(|filename| db.items(*filename))
            .collect();

        let birs = items
            .iter()
            .flat_map(|&item| item_op(item))
            .collect::<Vec<_>>();
        self.check_output_against_ref_file(format!("{birs:#?}"), bir_path, errors)?;

        Ok(())
    }

    async fn check_interpreted(
        &self,
        db: &dada_db::Db,
        filename: Filename,
        ref_path: &Path,
        errors: &mut Errors,
    ) -> eyre::Result<()> {
        let actual_output = match db.function_named(filename, "main") {
            Some(function) => {
                let kernel = BufferKernel::new();
                kernel.interpret_and_buffer(db, function).await;
                kernel.into_buffer()
            }
            None => {
                format!("no `main` function in `{}`", filename.as_str(db))
            }
        };
        self.check_output_against_ref_file(actual_output, ref_path, errors)?;
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
struct DiagnosticsDoNotMatch {
    expected: Vec<String>,
    actual: Vec<String>,
}

impl std::error::Error for DiagnosticsDoNotMatch {}

impl std::fmt::Display for DiagnosticsDoNotMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let expected: String = self
            .expected
            .iter()
            .flat_map(|e| vec![&e[..], "\n"])
            .collect();
        let actual: String = self
            .actual
            .iter()
            .flat_map(|e| vec![&e[..], "\n"])
            .collect();
        write!(
            f,
            "{}",
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header("from comments", "actual output")
        )
    }
}

#[derive(Debug)]
struct RefOutputDoesNotMatch {
    ref_path: PathBuf,
    expected: String,
    actual: String,
}

impl std::error::Error for RefOutputDoesNotMatch {}

impl std::fmt::Display for RefOutputDoesNotMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            similar::TextDiff::from_lines(&self.expected, &self.actual)
                .unified_diff()
                .header(&self.ref_path.display().to_string(), "actual output")
        )
    }
}

#[derive(Clone, Debug)]
struct ExpectedDiagnostic {
    start_line: u32,
    start_column: Option<u32>,
    end_line_column: Option<(u32, u32)>,
    severity: String,
    message: Regex,
}

/// Returns the diagnostics that we expect to see in the file, sorted by line number.
fn expected_diagnostics(path: &Path) -> eyre::Result<Vec<ExpectedDiagnostic>> {
    let file_contents = std::fs::read_to_string(path)?;

    let re = regex::Regex::new(r"^(?P<prefix>[^#]*)#! (?P<severity>[^ ]+) (?P<msg>.*)").unwrap();

    let mut last_code_line = 1;
    let mut result = vec![];
    for (line, line_number) in file_contents.lines().zip(1..) {
        if let Some(c) = re.captures(line) {
            let start_line = if c["prefix"].chars().all(char::is_whitespace) {
                // A comment alone on a line, like `#! ERROR ...`, will apply to the
                // last code line.
                last_code_line
            } else {
                // A comment at the end of a line, like `foo() #! ERROR`, applies to
                // that line.
                line_number
            };

            result.push(ExpectedDiagnostic {
                start_line,
                start_column: None,
                end_line_column: None,
                severity: c["severity"].to_string(),
                message: Regex::new(&c["msg"])?,
            });
        } else {
            last_code_line = line_number;
        }
    }
    Ok(result)
}

trait ActualDiagnostic: Clone {
    type Db: ?Sized;

    // Line number and column.
    fn start(&self, db: &Self::Db) -> (u32, u32);

    fn summary(&self, db: &Self::Db) -> String;

    fn severity(&self, db: &Self::Db) -> String;

    fn matches(&self, db: &Self::Db, expected: &ExpectedDiagnostic) -> bool;
}

impl ActualDiagnostic for dada_ir::diagnostic::Diagnostic {
    type Db = dada_db::Db;

    fn matches(&self, db: &Self::Db, expected: &ExpectedDiagnostic) -> bool {
        let (_, start, end) = db.line_columns(self.span);

        if start.line != expected.start_line {
            return false;
        }

        if let Some(start_column) = expected.start_column {
            if start.column != start_column {
                return false;
            }
        }

        if let Some((end_line, end_column)) = expected.end_line_column {
            if end_line != end.line || end_column != end.column {
                return false;
            }
        }

        // Check the severity against the one provided (if any).
        if expected.severity != self.severity(db) {
            return false;
        }

        expected.message.is_match(&self.message)
    }

    fn start(&self, db: &Self::Db) -> (u32, u32) {
        let (_, start, _) = db.line_columns(self.span);
        (start.line, start.column)
    }

    fn summary(&self, db: &Self::Db) -> String {
        let (filename, start, end) = db.line_columns(self.span);
        format!(
            " {}:{}:{}:{}:{}: {} {} [from db]",
            filename.as_str(db),
            start.line,
            start.column,
            end.line,
            end.column,
            self.severity(db),
            self.message
        )
    }

    fn severity(&self, _db: &Self::Db) -> String {
        format!("{:?}", self.severity).to_uppercase()
    }
}

impl ActualDiagnostic for Diagnostic {
    type Db = ();

    fn matches(&self, db: &(), expected: &ExpectedDiagnostic) -> bool {
        if self.range.start.line != expected.start_line {
            return false;
        }

        if let Some(start_column) = expected.start_column {
            if self.range.start.character != start_column {
                return false;
            }
        }

        if let Some((end_line, end_column)) = expected.end_line_column {
            if self.range.end.line != end_line {
                return false;
            }
            if self.range.end.character != end_column {
                return false;
            }
        }

        // Check the severity against the one provided (if any).
        if expected.severity != self.severity(db) {
            return false;
        }

        expected.message.is_match(&self.message)
    }

    fn start(&self, _db: &Self::Db) -> (u32, u32) {
        (self.range.start.line, self.range.start.character)
    }

    fn severity(&self, _db: &Self::Db) -> String {
        if let Some(s) = self.severity {
            format!("{s:?}").to_uppercase()
        } else {
            "(none)".to_string()
        }
    }

    fn summary(&self, db: &Self::Db) -> String {
        let (line, column) = self.start(db);
        format!(
            " {}:{}: {} {} [from LSP]",
            line,
            column,
            self.severity(db),
            self.message
        )
    }
}

impl ExpectedDiagnostic {
    fn summary(&self) -> String {
        format!(
            " {}: {} {} [expected]",
            self.start_line, self.severity, self.message
        )
    }
}

/// Remove system-specific absolute paths from output strings.
fn sanitize_output(output: String) -> eyre::Result<String> {
    let local_file_prefix = format!(
        r#""{}"#,
        match env::var("CARGO_MANIFEST_DIR") {
            Ok(v) => v,
            Err(_) => env::current_dir()?.display().to_string(),
        }
    );
    let replacement = r#""(local-file-prefix)"#;
    Ok(output.replace(&local_file_prefix, replacement))
}
