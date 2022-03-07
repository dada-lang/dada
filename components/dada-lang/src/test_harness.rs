use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::{env, fs};

use dada_execute::kernel::BufferKernel;
use dada_execute::machine::ProgramCounter;
use dada_ir::{filename::Filename, item::Item};
use eyre::Context;
use lsp_types::Diagnostic;
use regex::Regex;

mod heap_graph_query;
mod lsp_client;

#[derive(structopt::StructOpt)]
pub struct Options {
    /// Paths to directories and/or `.dada` files to test
    #[structopt(parse(from_os_str), default_value = "dada_tests")]
    dada_path: Vec<PathBuf>,

    /// Instead of validating `.ref` files, generate them
    #[structopt(long)]
    bless: bool,
}

impl Options {
    pub async fn main(&self, _crate_options: &crate::Options) -> eyre::Result<()> {
        let mut total = 0;
        let mut errors = Errors::default();
        let mut tests_with_fixmes = 0;

        if self.dada_path.is_empty() {
            eyre::bail!("no test paths given; try --dada-path");
        }

        const REF_EXTENSIONS: &[&str] = &["ref", "lsp", "bir", "validated", "syntax", "stdout"];

        for root in &self.dada_path {
            for entry in ignore::Walk::new(root) {
                let run_test = async {
                    let entry = entry?;
                    let path = entry.path();

                    if path.is_dir() {
                        return Ok(());
                    }

                    if let Some(ext) = path.extension() {
                        if ext == "dada" {
                            total += 1;
                            let fixmes = self
                                .test_dada_file(path)
                                .await
                                .with_context(|| format!("testing `{}`", path.display()))?;

                            if fixmes.is_empty() {
                                tracing::info!("test `{}` passed", path.display());
                            } else {
                                tests_with_fixmes += 1;

                                for fixme in fixmes {
                                    tracing::warn!(
                                        "test `{}` had expected bug: {}",
                                        path.display(),
                                        fixme
                                    );
                                }
                            }

                            return Ok(());
                        } else if REF_EXTENSIONS.iter().any(|e| *e == ext) {
                            // ignore ref files
                            if let Some(parent) = path.parent() {
                                let expected_dada_file = parent.with_extension("dada");
                                if !expected_dada_file.exists() {
                                    tracing::warn!(
                                        "found {:?}, but {:?} does not exist",
                                        path,
                                        expected_dada_file
                                    );
                                }
                            }
                            return Ok(());
                        } else if ext == "md" {
                            // allow .md files
                            return Ok(());
                        }
                    }

                    // Error out for random files -- I've frequently accidentally made
                    // tests with the extension `dad`, for example; but note that the
                    // directory walk obeys gitignore files, so things like emacs
                    // backup files will be skipped if the environment is configured
                    // appropriately.
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

        if tests_with_fixmes > 0 {
            tracing::info!("{tests_with_fixmes} test(s) encountered known bugs");
        }

        if num_errors == 0 {
            Ok(())
        } else {
            eyre::bail!("{} tests failed", num_errors)
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn test_dada_file(&self, path: &Path) -> eyre::Result<Vec<String>> {
        let expected_queries = &expected_queries(path)?;
        let expected_diagnostics = expected_diagnostics(path)?;
        let path_without_extension = path.with_extension("");
        fs::create_dir_all(&path_without_extension)?;
        self.test_dada_file_normal(
            &path_without_extension,
            &expected_diagnostics,
            expected_queries,
        )
        .await?;
        self.test_dada_file_in_ide(&path_without_extension, &expected_diagnostics)?;
        Ok(expected_diagnostics.fixmes)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn test_dada_file_normal(
        &self,
        path: &Path,
        expected_diagnostics: &ExpectedDiagnostics,
        expected_queries: &[Query],
    ) -> eyre::Result<()> {
        let mut db = dada_db::Db::default();
        let source_path = path.with_extension("dada");
        let contents = std::fs::read_to_string(&source_path)
            .with_context(|| format!("reading `{}`", &source_path.display()))?;
        let filename = dada_ir::filename::Filename::from(&db, &source_path);
        db.update_file(filename, contents);
        let diagnostics = db.diagnostics(filename);

        let mut errors = Errors::default();
        self.match_diagnostics_against_expectations(
            &db,
            &diagnostics,
            &expected_diagnostics.compile,
            &mut errors,
        )?;
        self.check_output_against_ref_file(
            dada_error_format::format_diagnostics_with_options(
                &db,
                &diagnostics,
                dada_error_format::FormatOptions::no_color(),
            )?,
            &path.join("compiler-output.ref"),
            &mut errors,
        )?;
        self.check_compiled(
            &db,
            &[filename],
            |item| db.debug_syntax_tree(item),
            &path.join("syntax.ref"),
        )?;
        self.check_compiled(
            &db,
            &[filename],
            |item| db.debug_validated_tree(item),
            &path.join("validated.ref"),
        )?;
        self.check_compiled(
            &db,
            &[filename],
            |item| db.debug_bir(item),
            &path.join("bir.ref"),
        )?;
        self.check_interpreted(
            &db,
            filename,
            &path.join("stdout.ref"),
            &expected_diagnostics.runtime,
            &expected_diagnostics.output,
            &mut errors,
        )
        .await?;

        for (query, query_index) in expected_queries.iter().zip(0..) {
            self.perform_query_on_db(&mut db, path, filename, query, query_index, &mut errors)
                .await?;
        }

        errors.into_result()
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn test_dada_file_in_ide(
        &self,
        path: &Path,
        expected_diagnostics: &ExpectedDiagnostics,
    ) -> eyre::Result<()> {
        let mut c = lsp_client::ChildSession::spawn();
        c.send_init()?;
        c.send_open(&path.with_extension("dada"))?;
        let diagnostics = c.receive_errors()?;

        let mut errors = Errors::default();
        self.match_diagnostics_against_expectations(
            &(),
            &diagnostics,
            &expected_diagnostics.compile,
            &mut errors,
        )?;
        self.maybe_bless_ref_file(format!("{:#?}", diagnostics), &path.join("lsp.ref"))?;
        errors.into_result()
    }

    async fn perform_query_on_db(
        &self,
        db: &mut dada_db::Db,
        path: &Path,
        filename: Filename,
        query: &Query,
        query_index: usize,
        errors: &mut Errors,
    ) -> eyre::Result<()> {
        match query.kind {
            QueryKind::HeapGraph => self
                .perform_heap_graph_query_on_db(db, path, query_index, filename, query, errors)
                .await
                .with_context(|| format!("heap query from line `{}`", query.line)),
        }
    }

    fn check_output_against_ref_file(
        &self,
        actual_output: String,
        ref_path: &Path,
        errors: &mut Errors,
    ) -> eyre::Result<()> {
        let sanitized_output = self.maybe_bless_ref_file(actual_output, ref_path)?;
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

    fn maybe_bless_ref_file(&self, actual_output: String, ref_path: &Path) -> eyre::Result<String> {
        let sanitized_output = sanitize_output(actual_output)?;
        self.maybe_bless_file(ref_path, &sanitized_output)?;
        Ok(sanitized_output)
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
        self.maybe_bless_ref_file(format!("{birs:#?}"), bir_path)?;

        Ok(())
    }

    async fn check_interpreted(
        &self,
        db: &dada_db::Db,
        filename: Filename,
        ref_path: &Path,
        expected_diagnostics: &[ExpectedDiagnostic],
        expected_outputs: &Option<Vec<ExpectedOutput>>,
        errors: &mut Errors,
    ) -> eyre::Result<()> {
        let mut diagnostics = vec![];
        let actual_output = match db.function_named(filename, "main") {
            Some(function) => {
                let mut kernel = BufferKernel::new().track_output_ranges(true);
                let res = kernel.interpret(db, function, vec![]).await;
                if let Err(err) = res {
                    match err.downcast_ref::<dada_execute::DiagnosticError>() {
                        Some(err) => {
                            diagnostics.push(err.diagnostic().clone());
                        }
                        None => {
                            eyre::bail!("unexpected runtime error type: {:?}", err);
                        }
                    }
                }

                if let Some(expected_outputs) = expected_outputs {
                    self.match_output_against_expectations(
                        db,
                        filename,
                        kernel.buffer_with_pcs(),
                        expected_outputs,
                        errors,
                    )?;
                }

                kernel.take_buffer()
            }
            None => {
                format!("no `main` function in `{}`", filename.as_str(db))
            }
        };
        self.match_diagnostics_against_expectations(
            db,
            &diagnostics,
            expected_diagnostics,
            errors,
        )?;
        self.check_output_against_ref_file(actual_output, ref_path, errors)?;
        Ok(())
    }

    fn match_output_against_expectations<'a>(
        &self,
        db: &dada_db::Db,
        filename: Filename,
        output_with_pcs: impl Iterator<Item = (&'a str, Option<ProgramCounter>)>,
        expected_outputs: &[ExpectedOutput],
        errors: &mut Errors,
    ) -> eyre::Result<()> {
        let mut actual_diffs = vec![];
        let mut expected_diffs = vec![];

        // First, collect each bit of output that came from each line into a map, in order.
        // Use a btreemap so that we will iterate over the keys in order.
        let mut output_by_line = BTreeMap::default();
        for (text, pc) in output_with_pcs {
            let pc = pc.ok_or_else(|| eyre::eyre!("untracked output `{}` from text", text))?;
            let span = pc.span(db);
            let (_, start_line_column, _) = db.line_columns(span);
            output_by_line
                .entry(start_line_column.line1())
                .or_insert(vec![])
                .push(text.to_string());
        }

        // Get the set of lines that either had output or expectations (in order).
        let line1s: BTreeSet<u32> = output_by_line
            .keys()
            .copied()
            .chain(expected_outputs.iter().map(|e| e.line1))
            .collect();

        let filename = filename.as_str(db);

        for line1 in line1s {
            let mut expected_on_this_line = expected_outputs
                .iter()
                .filter(|e| e.line1 == line1)
                .map(|e| &e.message)
                .peekable();
            let mut actual_on_this_line = output_by_line
                .get(&line1)
                .map(|v| &v[..])
                .unwrap_or(&[])
                .iter()
                .peekable();

            while let (Some(e), Some(a)) =
                (expected_on_this_line.peek(), actual_on_this_line.peek())
            {
                actual_diffs.push(format!(" {filename}:{line1}: {a:?}"));

                if e.is_match(a) {
                    // If the regex is a match, push the actual output so that the diff shows them the same.
                    expected_diffs.push(format!(" {filename}:{line1}: {a:?}"));
                } else {
                    // Else push the regex.
                    expected_diffs.push(format!(" {filename}:{line1}: something matching `{e:?}`"));
                }

                expected_on_this_line.next();
                actual_on_this_line.next();
            }

            for e in expected_on_this_line {
                expected_diffs.push(format!(" {filename}:{line1}: something matching `{e:?}`"));
            }

            for a in actual_on_this_line {
                actual_diffs.push(format!(" {filename}:{line1}: {a:?}"));
            }
        }

        if expected_diffs != actual_diffs {
            errors.push(OutputsDoNotMatch {
                expected: expected_diffs,
                actual: actual_diffs,
            });
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

/// Part of the code to check diagnostics: the diagnostics checker pushes
/// strings into these vectors so we can display a nice diff.
///
/// If we find what we expected, we push the same string into expected/actual.
///
/// If we don't, we push the regex we expected into one, and the actual output
/// into actual.
#[derive(Debug)]
struct DiagnosticsDoNotMatch {
    expected: Vec<String>,
    actual: Vec<String>,
}

impl std::error::Error for DiagnosticsDoNotMatch {}

impl std::fmt::Display for DiagnosticsDoNotMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_diff(&self.expected, &self.actual, f)
    }
}

/// Part of the code to check output: the output checker pushes
/// strings into these vectors so we can display a nice diff.
///
/// If we find what we expected, we push the same string into expected/actual.
///
/// If we don't, we push the regex we expected into one, and the actual output
/// into actual.
#[derive(Debug)]
struct OutputsDoNotMatch {
    expected: Vec<String>,
    actual: Vec<String>,
}

impl std::error::Error for OutputsDoNotMatch {}

impl std::fmt::Display for OutputsDoNotMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_diff(&self.expected, &self.actual, f)
    }
}

fn display_diff(
    expected: &[String],
    actual: &[String],
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    let expected: String = expected.iter().flat_map(|e| vec![&e[..], "\n"]).collect();
    let actual: String = actual.iter().flat_map(|e| vec![&e[..], "\n"]).collect();
    write!(
        f,
        "{}",
        similar::TextDiff::from_lines(&expected, &actual)
            .unified_diff()
            .header("from comments", "actual output")
    )
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

/// Test files have `#!` lines embedded in them indicating the
/// errors, warnings, and other diganostics we expect the compiler
/// to emit. Every diagnostic emitted by the compiler must have
/// a line like this or the test will fail.
#[derive(Clone, Debug)]
struct ExpectedDiagnostic {
    /// Line where the error is expected to start.
    start_line: u32,

    /// Start column, if given by the user.
    start_column: Option<u32>,

    /// End position, if given by the user.
    end_line_column: Option<(u32, u32)>,

    /// Expected severity ("ERROR", "WARNING", etc) of the diagnostic.
    severity: String,

    /// A regular expression given by the user that must match
    /// against the diagnostic.
    message: Regex,
}

/// Test files have `#! OUTPUT` lines that indicate output
/// that is expected from a particular line.
#[derive(Clone, Debug)]
struct ExpectedOutput {
    /// Line where the output should originate.
    line1: u32,

    /// A regular expression given by the user that must match
    /// against what was printed.
    message: Regex,
}

/// A query is indicated by a `#?   ^ QK` annotation. It means that,
/// on the preceding line, we should do some sort of interactive
/// query (specified by the `QK` string, see [`QueryKind`]) at the column
/// indicated by `^`. This could be a compilation
/// or runtime query. The results will be dumped into a file
/// but may also be queried with the regex in `message`.
#[derive(Clone, Debug)]
struct Query {
    line: u32,
    column: u32,
    kind: QueryKind,
    message: Regex,
}

/// Kinds of queries we can perform (see [`Query`])
#[derive(Clone, Debug)]
enum QueryKind {
    /// Interpret the code to this point and dump the heap-graph.
    HeapGraph,
}

/// There are both compile-time and runtime-emitted diagnostics
#[derive(Debug)]
struct ExpectedDiagnostics {
    compile: Vec<ExpectedDiagnostic>,
    runtime: Vec<ExpectedDiagnostic>,

    // If `None`, do not check the output.
    output: Option<Vec<ExpectedOutput>>,

    // Any `#! FIXME` annotations found
    fixmes: Vec<String>,
}

/// Returns the diagnostics that we expect to see in the file, sorted by line number.
fn expected_diagnostics(path: &Path) -> eyre::Result<ExpectedDiagnostics> {
    let file_contents = std::fs::read_to_string(path)?;

    let diagnostic_marker = regex::Regex::new(
        r"^(?P<prefix>[^#]*)#!\s*(?P<highlight>\^+)?\s*(?P<type>RUN)?\s*(?P<severity>ERROR|WARNING|INFO)\s*(?P<msg>.*)",
    )
    .unwrap();

    let output_marker = regex::Regex::new(r"^(?P<prefix>[^#]*)#!\s*OUTPUT\s+(?P<msg>.*)").unwrap();

    let fixme_issue_marker =
        regex::Regex::new(r"#!\s*FIXME\(#(?P<issue>[0-9]+)\): (?P<message>.+)").unwrap();
    let fixme_marker = regex::Regex::new(r"#!\s*FIXME: (?P<message>.+)").unwrap();

    let any_output_marker = regex::Regex::new(r"^(?P<prefix>[^#]*)#!\s*OUTPUT ANY").unwrap();

    let any_marker = regex::Regex::new(r"^[^#]*#!").unwrap();

    let mut last_code_line = 1;
    let mut compile_diagnostics = vec![];
    let mut runtime_diagnostics = vec![];
    let mut output = vec![];
    let mut fixmes = vec![];
    let mut any_output_marker_seen = None;
    for (line, line_number) in file_contents.lines().zip(1..) {
        if let Some(c) = diagnostic_marker.captures(line) {
            let start_line = if c["prefix"].chars().all(char::is_whitespace) {
                // A comment alone on a line, like `#! ERROR ...`, will apply to the
                // last code line.
                last_code_line
            } else {
                // A comment at the end of a line, like `foo() #! ERROR`, applies to
                // that line.
                line_number
            };

            let highlight = c.name("highlight");
            let start_column = highlight.map(|m| m.start() as u32 + 1);
            let end_line_column = highlight.map(|m| (start_line, m.end() as u32 + 1));
            let severity = c["severity"].to_string();
            let message = Regex::new(&c["msg"])?;
            let expected = ExpectedDiagnostic {
                start_line,
                start_column,
                end_line_column,
                severity,
                message,
            };
            let type_ = c.name("type").map_or("COMPILE", |m| m.as_str());

            match type_ {
                "COMPILE" => {
                    compile_diagnostics.push(expected);
                }
                "RUN" => {
                    runtime_diagnostics.push(expected);
                }
                wrong => {
                    eyre::bail!("unexpected diagnostic type {} in {:?}", wrong, path);
                }
            }
        } else if any_output_marker.is_match(line) {
            any_output_marker_seen = Some(line_number);
        } else if let Some(c) = output_marker.captures(line) {
            let line1 = if c["prefix"].chars().all(char::is_whitespace) {
                // A comment alone on a line, like `#! ERROR ...`, will apply to the
                // last code line.
                last_code_line
            } else {
                // A comment at the end of a line, like `foo() #! ERROR`, applies to
                // that line.
                line_number
            };
            let message = Regex::new(&c["msg"])?;
            output.push(ExpectedOutput { line1, message });
        } else if let Some(c) = fixme_issue_marker.captures(line) {
            fixmes.push(format!("#{}", &c["issue"]));
        } else if let Some(c) = fixme_marker.captures(line) {
            fixmes.push(c["message"].trim().to_string());
        } else if any_marker.is_match(line) {
            eyre::bail!(
                "`#!` marker on line {} doesn't have expected form",
                line_number
            )
        } else {
            last_code_line = line_number;
        }
    }

    if let Some(any_line) = any_output_marker_seen {
        if let Some(o) = output.get(0) {
            eyre::bail!(
                "both 'OUTPUT ANY' (on line {}) and specific output (e.g. on line {}) found",
                any_line,
                o.line1
            );
        }
    }

    Ok(ExpectedDiagnostics {
        compile: compile_diagnostics,
        runtime: runtime_diagnostics,
        output: if any_output_marker_seen.is_some() {
            None
        } else {
            Some(output)
        },
        fixmes,
    })
}

/// Searches for a `#?` annotation, which indicates that we want to do a
/// query at a particular point.
fn expected_queries(path: &Path) -> eyre::Result<Vec<Query>> {
    let file_contents = std::fs::read_to_string(path)?;

    // The notation #?    ^ indicates the line/column pictorially
    let query_at_re =
        regex::Regex::new(r"^[^#]*#\?\s*(?P<pointer>\^)\s*(?P<kind>[^\s]*)\s*(?P<msg>.*)").unwrap();

    // The notation #? @ 1:1 indicates the line/column by number
    let query_lc_re = regex::Regex::new(
        r"^[^#]*#\?\s*@\s*(?P<line>[+-]?\d+):(?P<column>\d+)\s*(?P<kind>[^\s]*)\s*(?P<msg>.*)",
    )
    .unwrap();

    // The notation #? @ 1:1 indicates the line/column by number
    let query_any_re = regex::Regex::new(r"^[^#]*#\?").unwrap();

    let comment_line_re = regex::Regex::new(r"^\s*#").unwrap();

    let mut last_code_line = 1;
    let mut result = vec![];
    for (line, line_number) in file_contents.lines().zip(1..) {
        if let Some(c) = query_at_re.captures(line) {
            // The column comes from the position of the `^`.
            let column = u32::try_from(c.name("pointer").unwrap().start() + 1).unwrap();

            let query_kind = match &c["kind"] {
                "HeapGraph" => QueryKind::HeapGraph,
                k => eyre::bail!("unexpected query kind `{}` on line {}", k, line_number),
            };

            result.push(Query {
                line: last_code_line,
                column,
                kind: query_kind,
                message: Regex::new(&c["msg"])?,
            });
            tracing::debug!("query {:?} found", result.last());
        } else if let Some(c) = query_lc_re.captures(line) {
            // The column comes from the position of the `^`.
            let given_line_number: u32 = parse_line_number(line_number, &c["line"])?;
            let given_column_number: u32 = str::parse(&c["column"])
                .with_context(|| format!("in query on line {}", line_number))?;

            let query_kind = match &c["kind"] {
                "HeapGraph" => QueryKind::HeapGraph,
                k => eyre::bail!("unexpected query kind `{}` on line {}", k, line_number),
            };

            result.push(Query {
                line: given_line_number,
                column: given_column_number,
                kind: query_kind,
                message: Regex::new(&c["msg"])?,
            });
            tracing::debug!("query {:?} found", result.last());
        } else if query_any_re.is_match(line) {
            eyre::bail!(
                "query `#?` on line {} doesn't match any known template",
                line_number
            );
        }

        if !comment_line_re.is_match(line) {
            last_code_line = line_number;
        }
    }
    Ok(result)
}

fn parse_line_number(current_line_number: u32, line: &str) -> eyre::Result<u32> {
    let (sign, number) = if let Some(suffix) = line.strip_prefix('+') {
        // Below current line
        (1, suffix)
    } else if let Some(suffix) = line.strip_prefix('-') {
        // Above current line
        (-1, suffix)
    } else {
        // Absolute
        (0, line)
    };

    let parsed: u32 =
        str::parse(number).with_context(|| format!("in query on line {}", current_line_number))?;
    #[allow(clippy::comparison_chain)]
    Ok(if sign == 0 {
        parsed
    } else if sign > 0 {
        current_line_number + parsed
    } else {
        current_line_number - parsed
    })
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

        if start.line1() != expected.start_line {
            return false;
        }

        if let Some(start_column) = expected.start_column {
            if start.column1() != start_column {
                return false;
            }
        }

        if let Some((end_line, end_column)) = expected.end_line_column {
            if end_line != end.line1() || end_column != end.column1() {
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
        (start.line1(), start.column1())
    }

    fn summary(&self, db: &Self::Db) -> String {
        let (filename, start, end) = db.line_columns(self.span);
        format!(
            " {}:{}:{}:{}:{}: {} {} [from db]",
            filename.as_str(db),
            start.line1(),
            start.column1(),
            end.line1(),
            end.column1(),
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
        let column = if let Some(start_column) = self.start_column {
            if let Some((end_line, end_column)) = self.end_line_column {
                format!("{start_column}:{end_line}:{end_column}")
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };

        format!(
            " {}:{column} {} {} [expected]",
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
