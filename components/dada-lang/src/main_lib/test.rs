use std::{
    panic::AssertUnwindSafe,
    path::{Path, PathBuf},
};

use dada_compiler::{Compiler, RealFs};
use dada_ir_ast::diagnostic::{Diagnostic, Level};
use dada_util::{Fallible, bail};
use expected::{ExpectedDiagnostic, Probe};
use indicatif::ProgressBar;
use panic_hook::CapturedPanic;
use rayon::prelude::*;
use walkdir::WalkDir;

use crate::{GlobalOptions, TestOptions};

use super::Main;

mod expected;
mod spec_validation;
mod timeout_warning;

#[derive(thiserror::Error, Debug)]
#[error("{} test failure(s)", failed_tests.len())]
struct FailedTests {
    failed_tests: Vec<FailedTest>,
}

#[derive(Debug)]
struct FailedTest {
    path: PathBuf,
    full_compiler_output: String,
    failures: Vec<Failure>,
}

#[derive(Debug)]
enum TestResult {
    /// Test passed as expected
    Passed,
    /// Test failed as expected (not FIXME)
    Failed(FailedTest),
    /// FIXME test failed as expected
    FixmeFailed(FailedTest),
}

#[derive(Debug)]
enum Failure {
    UnexpectedDiagnostic(Diagnostic),
    MultipleMatches(ExpectedDiagnostic, Diagnostic),
    MissingDiagnostic(ExpectedDiagnostic),
    InternalCompilerError(Option<CapturedPanic>),

    /// A test marked as FIXME did not fail
    FixmePassed,

    /// The probe at the given location did not yield the expected result.
    Probe {
        /// Probe performed
        probe: Probe,

        /// Actual result returned
        actual: String,
    },

    /// Invalid spec reference in #:spec comment
    InvalidSpecReference(String),

    /// Auxiliary file at `path` did not have expected contents.
    ///
    /// See `diff`.
    ///
    /// You can auto-update these files by setting `UPDATE_EXPECT=1`.
    Auxiliary {
        kind: String,
        ref_path: PathBuf,
        txt_path: PathBuf,
        diff: String,
    },
}

impl Failure {}

mod panic_hook;

impl Main {
    pub(super) fn test(&mut self, mut options: TestOptions) -> Fallible<()> {
        let tests = if options.inputs.is_empty() {
            self.assemble_tests(&["tests"], &mut false)?
        } else {
            self.assemble_tests(&options.inputs, &mut options.verbose)?
        };

        let progress_bar = ProgressBar::new(tests.len() as u64);

        let test_results: Vec<Fallible<TestResult>> = panic_hook::recording_panics(|| {
            if options.verbose {
                tests
                    .iter()
                    .map(|input| self.run_test_with_progress(&options, input, &progress_bar))
                    .collect()
            } else {
                tests
                    .par_iter()
                    .map(|input| self.run_test_with_progress(&options, input, &progress_bar))
                    .collect()
            }
        });

        let mut failed_tests = vec![];
        let mut fixme_failed_tests = vec![];
        for test_result in test_results {
            match test_result? {
                TestResult::Passed => {}
                TestResult::Failed(failed_test) => failed_tests.push(failed_test),
                TestResult::FixmeFailed(failed_test) => fixme_failed_tests.push(failed_test),
            }
        }

        if failed_tests.len() == 1 {
            for failed_test in &failed_tests {
                let test_report = std::fs::read_to_string(failed_test.test_report_path())?;
                progress_bar.println(test_report);
            }
        }

        let total_passed = tests.len() - failed_tests.len() - fixme_failed_tests.len();
        if failed_tests.is_empty() {
            let message = if fixme_failed_tests.is_empty() {
                format!("All {} tests passed", tests.len())
            } else {
                format!(
                    "{} tests passed, {} FIXME tests failed (ignored)",
                    total_passed,
                    fixme_failed_tests.len()
                )
            };
            progress_bar.println(message);
            progress_bar.finish();

            Ok(())
        } else {
            // Include unexpected passing FIXME tests in the error count
            let message = format!("{} test failure(s)", failed_tests.len());
            progress_bar.println(message);
            progress_bar.finish();

            Err(FailedTests { failed_tests }.into())
        }
    }

    fn assemble_tests(
        &self,
        inputs: &[impl AsRef<Path>],
        verbose: &mut bool,
    ) -> Fallible<Vec<PathBuf>> {
        let mut result = vec![];

        // If there is exactly one input specified and it is a file (not a directory),
        // set verbose to true.
        if inputs.len() == 1 && inputs[0].as_ref().is_file() {
            *verbose = true;
        }

        for input in inputs {
            let input: &Path = input.as_ref();

            if !input.exists() {
                bail!("test path '{}' does not exist", input.display());
            } else if is_dada_file(input) {
                result.push(input.to_path_buf());
            } else if input.is_dir() {
                for e in WalkDir::new(input) {
                    let e = e?;
                    if is_dada_file(e.path()) {
                        result.push(e.into_path());
                    }
                }
            } else {
                bail!(
                    "input path '{}' is neither a .dada file nor a directory",
                    input.display()
                );
            }
        }

        Ok(result)
    }

    fn run_test_with_progress(
        &self,
        options: &TestOptions,
        input: &Path,
        progress_bar: &ProgressBar,
    ) -> Fallible<TestResult> {
        timeout_warning::timeout_warning(input, || {
            if options.verbose {
                progress_bar.println(format!("{}: beginning test", input.display(),));
            }

            let result = self.run_test(input);
            match &result {
                Ok(TestResult::Passed) => {}
                Ok(TestResult::Failed(error)) => {
                    progress_bar.println(format!("{}: {}", input.display(), error.summarize()));
                    if options.verbose {
                        let test_report = std::fs::read_to_string(error.test_report_path())?;
                        progress_bar.println(test_report);
                    }
                }
                Ok(TestResult::FixmeFailed(error)) => {
                    progress_bar.println(format!(
                        "{}: FIXME test failed (as expected), {}",
                        input.display(),
                        error.summarize()
                    ));
                    if options.verbose {
                        let test_report = std::fs::read_to_string(error.test_report_path())?;
                        progress_bar.println(test_report);
                    }
                }
                Err(error) => progress_bar.println(format!(
                    "{}: test harness errored, {}",
                    input.display(),
                    error
                )),
            }
            progress_bar.inc(1);
            result
        })
    }

    /// Run a single test found at the given path.
    ///
    /// # Returns
    ///
    /// * `Err(e)` for some failure in the test harness itself.
    /// * `Ok(result)` with the test result.
    fn run_test(&self, input: &Path) -> Fallible<TestResult> {
        assert!(is_dada_file(input));
        let mut compiler = Compiler::new(RealFs::default(), None);

        // Run the test and capture panics
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            let source_file = compiler.load_source_file(input)?;
            let expectations = expected::TestExpectations::new(&compiler, source_file)?;
            expectations.compare(&mut compiler)
        }));

        let (failed_test, is_fixme) = match result {
            // No panic occurred: just propagate test harness errors and continue
            Ok(r) => r?,

            // Panic occurred: convert that into a test failure
            Err(_unwound) => {
                let captured_panic = panic_hook::captured_panic();
                (Some(FailedTest::ice(input, captured_panic)), false)
            }
        };

        match (failed_test, is_fixme) {
            (None, false) => {
                delete_test_report(input)?;
                Ok(TestResult::Passed)
            }
            (None, true) => {
                let failed_test = FailedTest::fixme_passed(input);
                failed_test.generate_test_report(&compiler)?;
                Ok(TestResult::Failed(failed_test))
            }
            (Some(failed_test), is_fixme) => {
                failed_test.generate_test_report(&compiler)?;
                if is_fixme {
                    Ok(TestResult::FixmeFailed(failed_test))
                } else {
                    Ok(TestResult::Failed(failed_test))
                }
            }
        }
    }
}

fn is_dada_file(input: &Path) -> bool {
    input.is_file() && input.extension().map(|e| e == "dada").unwrap_or(false)
}

impl FailedTest {
    fn ice(path: &Path, captured_panic: Option<CapturedPanic>) -> Self {
        FailedTest {
            path: path.to_path_buf(),
            full_compiler_output: "(Internal Compiler Error)\n".to_string(),
            failures: vec![Failure::InternalCompilerError(captured_panic)],
        }
    }

    fn fixme_passed(path: &Path) -> Self {
        FailedTest {
            path: path.to_path_buf(),
            full_compiler_output: "FIXME test passed!\n".to_string(),
            failures: vec![Failure::FixmePassed],
        }
    }

    fn test_report_path(&self) -> PathBuf {
        test_report_path(&self.path)
    }

    fn summarize(&self) -> String {
        format!(
            "{} failures, see {}",
            self.failures.len(),
            self.test_report_path().display()
        )
    }

    fn report(&self, db: &dyn crate::Db) -> Fallible<String> {
        use std::fmt::Write;
        let opts = GlobalOptions::test_options();

        let mut result = String::new();

        writeln!(result, "Test failed: {}", self.path.display())?;

        writeln!(result)?;
        writeln!(
            result,
            "[Test file](./{})",
            self.path.file_name().unwrap().to_string_lossy()
        )?;
        writeln!(result)?;

        writeln!(result)?;
        writeln!(result, "# Compiler output")?;
        writeln!(result)?;
        writeln!(result, "```\n{}```", self.full_compiler_output)?;

        for failure in &self.failures {
            match failure {
                Failure::UnexpectedDiagnostic(diagnostic) => {
                    writeln!(result)?;
                    writeln!(result, "# Unexpected diagnostic")?;
                    writeln!(result)?;

                    let render = diagnostic.render(db, &opts.render_opts());
                    writeln!(result, "```\n{render}\n```")?;
                    writeln!(result)?;
                    writeln!(result, "```\n{diagnostic:#?}\n```\n")?;
                }
                Failure::MultipleMatches(expected, actual) => {
                    writeln!(result)?;
                    writeln!(result, "# Multiple matches for expected diagnostic")?;
                    writeln!(result)?;

                    writeln!(result, "Diagnostic:")?;
                    let render = actual.render(db, &opts.render_opts());
                    writeln!(result, "```\n{render}\n```")?;
                    writeln!(result)?;
                    writeln!(result, "```\n{actual:#?}\n```\n")?;
                    writeln!(result)?;
                    writeln!(result, "Expected diagnostic that matched multiple times:")?;
                    writeln!(result, "```\n{expected:#?}\n```")?;
                }
                Failure::MissingDiagnostic(expected) => {
                    writeln!(result)?;
                    writeln!(result, "# Missing expected diagnostic")?;
                    writeln!(result)?;

                    // Format this nicely
                    let annotation_span = expected.annotation_span.into_span(db);
                    let diagnostic = Diagnostic::new(
                        db,
                        Level::Error,
                        annotation_span,
                        "missing expected diagnostic",
                    )
                    .label(
                        db,
                        Level::Error,
                        annotation_span,
                        "this diagnostic was never reported",
                    );
                    let render = diagnostic.render(db, &opts.render_opts());
                    writeln!(result, "```\n{render}\n```")?;
                }
                Failure::Auxiliary {
                    kind,
                    ref_path,
                    txt_path,
                    diff,
                } => {
                    writeln!(result)?;
                    writeln!(result, "# {kind} did not match")?;
                    writeln!(result)?;
                    writeln!(
                        result,
                        "[Reference]({})",
                        self.relativize(&self.path, ref_path).display()
                    )?;
                    writeln!(
                        result,
                        "[Actual]({})",
                        self.relativize(&self.path, txt_path).display()
                    )?;
                    writeln!(result)?;

                    writeln!(result, "Diff:")?;
                    writeln!(result, "```diff\n{diff}\n```")?;
                }
                Failure::InternalCompilerError(captured_panic) => {
                    writeln!(result)?;
                    writeln!(result, "# Internal compiler error")?;
                    writeln!(result)?;
                    if let Some(captured_panic) = captured_panic {
                        writeln!(result, "{}", captured_panic.render())?;
                    } else {
                        writeln!(result, "No details available. :(")?;
                    }
                }
                Failure::Probe { probe, actual } => {
                    writeln!(result)?;
                    writeln!(result, "# Probe return unexpected result")?;
                    writeln!(result)?;

                    let (probe_line, probe_start_col) =
                        probe.span.source_file.line_col(db, probe.span.start);
                    let (probe_end_line, probe_end_col) =
                        probe.span.source_file.line_col(db, probe.span.end);
                    assert_eq!(
                        probe_line, probe_end_line,
                        "multiline probe not currently possible"
                    );

                    writeln!(
                        result,
                        "Probe location: {u}:{l}:{c}:{l}:{e}",
                        u = probe.span.source_file.url_display(db),
                        l = probe_line.as_u32() + 1,
                        c = probe_start_col.as_u32() + 1,
                        e = probe_end_col.as_u32() + 1,
                    )?;
                    writeln!(result, "Probe expected: {e}", e = probe.message)?;
                    writeln!(result, "Probe got: {actual}")?;

                    let file_text = probe.span.source_file.contents_if_ok(db);
                    let line_range = probe.span.source_file.line_range(db, probe_line);
                    if let Some(line_text) =
                        file_text.get(line_range.start.as_usize()..line_range.end.as_usize())
                    {
                        writeln!(result)?;
                        writeln!(result, "```")?;
                        write!(result, "{line_text}")?;
                        writeln!(
                            result,
                            "{s}{c} probe `{k:?}` expected `{e}`, got `{a}`",
                            s = std::iter::repeat_n(' ', probe_start_col.as_usize())
                                .collect::<String>(),
                            c = std::iter::repeat_n(
                                '^',
                                (probe_end_col - probe_start_col).as_usize()
                            )
                            .collect::<String>(),
                            k = probe.kind,
                            e = probe.message,
                            a = actual,
                        )?;
                        writeln!(result, "```")?;
                        writeln!(result)?;
                    }
                }
                Failure::FixmePassed => {
                    writeln!(result)?;
                    writeln!(result, "# Test marked as FIXME and yet it passed")?;
                    writeln!(result)?;
                    writeln!(result, "Perhaps the bug was fixed?")?;
                }
                Failure::InvalidSpecReference(spec_ref) => {
                    writeln!(result)?;
                    writeln!(result, "# Invalid spec reference")?;
                    writeln!(result)?;
                    writeln!(result, "The spec reference `{}` does not exist in the spec mdbook.", spec_ref)?;
                    writeln!(result, "Check the spec files in `spec/src/` for valid `r[...]` labels.")?;
                }
            }
        }

        Ok(result)
    }

    fn relativize<'aux>(&self, test_path: &Path, aux_path: &'aux Path) -> &'aux Path {
        if let Some(dir) = test_path.parent() {
            aux_path.strip_prefix(dir).unwrap_or(aux_path)
        } else {
            aux_path
        }
    }

    fn generate_test_report(&self, db: &dyn crate::Db) -> Fallible<()> {
        std::fs::write(test_report_path(&self.path), self.report(db)?)?;
        Ok(())
    }
}

fn delete_test_report(path: &Path) -> Fallible<()> {
    let path = test_report_path(path);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

fn test_report_path(path: &Path) -> PathBuf {
    path.with_extension("test-report.md")
}
