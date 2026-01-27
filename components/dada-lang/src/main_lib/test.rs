use std::{
    panic::AssertUnwindSafe,
    path::{Path, PathBuf},
    time::Instant,
};

use dada_compiler::{Compiler, RealFs};
use dada_ir_ast::diagnostic::{Diagnostic, Level};
use dada_util::{Fallible, bail};
use expected::{ExpectedDiagnostic, Probe};
use indicatif::ProgressBar;
use panic_hook::CapturedPanic;
use rayon::prelude::*;
use serde::Serialize;
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

/// Enhanced test result with timing and metadata for porcelain output
#[derive(Debug)]
struct DetailedTestResult {
    path: PathBuf,
    result: TestResult,
    duration_ms: u64,
    annotations: Vec<String>,
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

/// JSON output structures for --porcelain mode
#[derive(Serialize)]
struct PorcelainOutput {
    summary: PorcelainSummary,
    tests: Vec<PorcelainTest>,
}

#[derive(Serialize)]
struct PorcelainSummary {
    total: usize,
    passed: usize,
    failed: usize,
    duration_ms: u64,
}

#[derive(Serialize)]
struct PorcelainTest {
    path: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    annotations: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    suggestion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
    duration_ms: u64,
}

mod panic_hook;

/// Trait for formatting test output in different modes
trait TestOutputFormatter: Sync + Send {
    fn format_results(&self, results: Vec<DetailedTestResult>, total_duration_ms: u64) -> Fallible<()>;
    fn show_progress(&self, path: &Path, result: &DetailedTestResult, verbose: bool);
}

/// Formats output for human consumption with progress bars
struct RegularFormatter {
    progress_bar: ProgressBar,
}

impl TestOutputFormatter for RegularFormatter {
    fn show_progress(&self, path: &Path, result: &DetailedTestResult, verbose: bool) {
        if verbose {
            self.progress_bar.println(format!("{}: beginning test", path.display()));
            match &result.result {
                TestResult::Passed => {}
                TestResult::Failed(error) => {
                    self.progress_bar.println(format!("{}: {}", path.display(), error.summarize()));
                }
                TestResult::FixmeFailed(error) => {
                    self.progress_bar.println(format!("{}: {} (FIXME)", path.display(), error.summarize()));
                }
            }
        }
        
        // Increment progress bar after each test
        self.progress_bar.inc(1);
    }

    fn format_results(&self, results: Vec<DetailedTestResult>, _total_duration_ms: u64) -> Fallible<()> {
        let mut failed_tests = vec![];
        let mut fixme_failed_tests = vec![];
        
        let total_tests = results.len();
        for detailed_result in results {
            match detailed_result.result {
                TestResult::Passed => {}
                TestResult::Failed(failed_test) => failed_tests.push(failed_test),
                TestResult::FixmeFailed(failed_test) => fixme_failed_tests.push(failed_test),
            }
        }

        if failed_tests.len() == 1 {
            for failed_test in &failed_tests {
                let test_report = std::fs::read_to_string(failed_test.test_report_path())?;
                self.progress_bar.println(test_report);
            }
        }

        let total_passed = total_tests - failed_tests.len() - fixme_failed_tests.len();
        
        if failed_tests.is_empty() {
            let message = if fixme_failed_tests.is_empty() {
                format!("All {} tests passed", total_tests)
            } else {
                format!(
                    "{} tests passed, {} FIXME tests failed (ignored)",
                    total_passed,
                    fixme_failed_tests.len()
                )
            };
            self.progress_bar.println(message);
            self.progress_bar.finish();
            Ok(())
        } else {
            let message = format!("{} test failure(s)", failed_tests.len());
            self.progress_bar.println(message);
            self.progress_bar.finish();
            Err(FailedTests { failed_tests }.into())
        }
    }
}

/// Formats output as machine-readable JSON
struct PorcelainFormatter;

impl TestOutputFormatter for PorcelainFormatter {
    fn show_progress(&self, _path: &Path, _result: &DetailedTestResult, _verbose: bool) {
        // No progress output for porcelain mode
    }

    fn format_results(&self, results: Vec<DetailedTestResult>, total_duration_ms: u64) -> Fallible<()> {
        let mut porcelain_tests = Vec::new();
        let mut failed_count = 0;

        for detailed_result in &results {
            let porcelain_test = convert_to_porcelain_test(detailed_result);
            if porcelain_test.status == "fail" {
                failed_count += 1;
            }
            porcelain_tests.push(porcelain_test);
        }

        let passed_count = results.len() - failed_count;

        let output = PorcelainOutput {
            summary: PorcelainSummary {
                total: results.len(),
                passed: passed_count,
                failed: failed_count,
                duration_ms: total_duration_ms,
            },
            tests: porcelain_tests,
        };

        println!("{}", serde_json::to_string_pretty(&output)?);

        if failed_count > 0 {
            // Create dummy failed tests for error handling
            let failed_tests: Vec<FailedTest> = output.tests
                .iter()
                .filter(|t| t.status == "fail")
                .map(|t| FailedTest {
                    path: PathBuf::from(&t.path),
                    full_compiler_output: t.details.clone().unwrap_or_default(),
                    failures: vec![], // We don't need detailed failures for porcelain mode
                })
                .collect();
            Err(FailedTests { failed_tests }.into())
        } else {
            Ok(())
        }
    }
}

impl Main {
    pub(super) fn test(&mut self, mut options: TestOptions) -> Fallible<()> {
        let tests = if options.inputs.is_empty() {
            self.assemble_tests(&["tests"], &mut false)?
        } else {
            self.assemble_tests(&options.inputs, &mut options.verbose)?
        };

        let start_time = Instant::now();

        // Create appropriate formatter
        let formatter: Box<dyn TestOutputFormatter> = if options.porcelain {
            Box::new(PorcelainFormatter)
        } else {
            Box::new(RegularFormatter {
                progress_bar: ProgressBar::new(tests.len() as u64),
            })
        };

        // Run tests
        let test_results: Vec<Fallible<DetailedTestResult>> = panic_hook::recording_panics(|| {
            let runner = |input: &Path| -> Fallible<DetailedTestResult> {
                let result = self.run_test(input)?;
                formatter.show_progress(input, &result, options.verbose);
                Ok(result)
            };

            if options.verbose {
                tests.iter().map(|input| runner(input)).collect()
            } else {
                tests.par_iter().map(|input| runner(input)).collect()
            }
        });

        // Collect results
        let results: Result<Vec<DetailedTestResult>, _> = test_results.into_iter().collect();
        let results = results?;

        let total_duration = start_time.elapsed().as_millis() as u64;
        formatter.format_results(results, total_duration)
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

    /// Run a single test found at the given path.
    ///
    /// # Returns
    ///
    /// * `Err(e)` for some failure in the test harness itself.
    /// * `Ok(result)` with the detailed test result including timing and annotations.
    fn run_test(&self, input: &Path) -> Fallible<DetailedTestResult> {
        let start_time = Instant::now();
        
        assert!(is_dada_file(input));
        let mut compiler = Compiler::new(RealFs::default(), None);

        // Get test annotations and run the test
        let source_file = compiler.load_source_file(input)?;
        let expectations = expected::TestExpectations::new(&compiler, source_file)?;
        let annotations = extract_annotations(&expectations);
        
        // Run the test and capture panics
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            expectations.compare(&mut compiler)
        }));

        let duration_ms = start_time.elapsed().as_millis() as u64;

        let test_result = match result {
            Ok(r) => {
                let (failed_test, is_fixme) = r?;
                match (failed_test, is_fixme) {
                    (None, false) => {
                        delete_test_report(input)?;
                        TestResult::Passed
                    }
                    (None, true) => {
                        let failed_test = FailedTest::fixme_passed(input);
                        failed_test.generate_test_report(&compiler)?;
                        TestResult::Failed(failed_test)
                    }
                    (Some(failed_test), is_fixme) => {
                        failed_test.generate_test_report(&compiler)?;
                        if is_fixme {
                            TestResult::FixmeFailed(failed_test)
                        } else {
                            TestResult::Failed(failed_test)
                        }
                    }
                }
            }
            Err(_unwound) => {
                let captured_panic = panic_hook::captured_panic();
                let failed_test = FailedTest::ice(input, captured_panic);
                failed_test.generate_test_report(&compiler)?;
                TestResult::Failed(failed_test)
            }
        };

        Ok(DetailedTestResult {
            path: input.to_path_buf(),
            result: test_result,
            duration_ms,
            annotations,
        })
    }

}

fn extract_annotations(expectations: &expected::TestExpectations) -> Vec<String> {
    let mut annotations = Vec::new();
    
    // Check the TestExpectations struct fields and convert to string annotations
    if expectations.fn_asts() {
        annotations.push("#:fn_asts".to_string());
    }
    if !expectations.codegen() {
        annotations.push("#:skip_codegen".to_string());
    }
    if expectations.fixme() {
        annotations.push("#:FIXME".to_string());
    }
    
    // Add spec references
    for spec_ref in expectations.spec_refs() {
        annotations.push(format!("#:spec {}", spec_ref));
    }
    
    annotations
}

fn convert_to_porcelain_test(detailed_result: &DetailedTestResult) -> PorcelainTest {
    let path = detailed_result.path.to_string_lossy().to_string();
    
    match &detailed_result.result {
        TestResult::Passed => PorcelainTest {
            path,
            status: "pass".to_string(),
            reason: None,
            annotations: detailed_result.annotations.clone(),
            suggestion: None,
            details: None,
            duration_ms: detailed_result.duration_ms,
        },
        TestResult::Failed(failed_test) => {
            let (reason, suggestion, details) = analyze_failure(failed_test);
            PorcelainTest {
                path,
                status: "fail".to_string(),
                reason: Some(reason),
                annotations: detailed_result.annotations.clone(),
                suggestion: Some(suggestion),
                details: Some(details),
                duration_ms: detailed_result.duration_ms,
            }
        },
        TestResult::FixmeFailed(_failed_test) => PorcelainTest {
            path,
            status: "pass".to_string(), // FIXME failures are treated as expected (passed)
            reason: None,
            annotations: detailed_result.annotations.clone(),
            suggestion: None,
            details: None,
            duration_ms: detailed_result.duration_ms,
        },
    }
}

fn analyze_failure(failed_test: &FailedTest) -> (String, String, String) {
    // Simplified approach: always point to test report for detailed guidance
    let test_report_path = failed_test.test_report_path();
    let suggestion = format!("Consult {} for details and guidance", test_report_path.display());
    
    (
        "test_failure".to_string(),
        suggestion,
        "See test report for detailed analysis and next steps".to_string(),
    )
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
                    writeln!(result, "Check the spec files in `spec/src/` for valid `:::\\{{spec}}` directives.")?;
                }
            }
        }

        // Add intelligent guidance section
        self.add_guidance_section(&mut result)?;

        Ok(result)
    }

    fn add_guidance_section(&self, result: &mut String) -> Fallible<()> {
        use std::fmt::Write;
        
        // Count different types of failures to provide targeted guidance
        let mut unexpected_diagnostics = 0;
        let mut missing_diagnostics = 0;
        let mut multiple_matches = 0;
        let mut auxiliary_failures = 0;
        let mut ice_failures = 0;
        let mut spec_failures = 0;
        let mut fixme_passed = 0;
        
        for failure in &self.failures {
            match failure {
                Failure::UnexpectedDiagnostic(_) => unexpected_diagnostics += 1,
                Failure::MissingDiagnostic(_) => missing_diagnostics += 1,
                Failure::MultipleMatches(_, _) => multiple_matches += 1,
                Failure::Auxiliary { .. } => auxiliary_failures += 1,
                Failure::InternalCompilerError(_) => ice_failures += 1,
                Failure::InvalidSpecReference(_) => spec_failures += 1,
                Failure::FixmePassed => fixme_passed += 1,
                _ => {}
            }
        }
        
        writeln!(result)?;
        writeln!(result, "# 🎯 Next Steps")?;
        writeln!(result)?;
        
        // Provide specific guidance based on failure types
        if unexpected_diagnostics > 0 || missing_diagnostics > 0 || multiple_matches > 0 {
            writeln!(result, "## Diagnostic Expectation Issues")?;
            writeln!(result)?;
            writeln!(result, "This test has diagnostic-related failures. Choose one approach:")?;
            writeln!(result)?;
            writeln!(result, "**Option 1: Add diagnostic annotations** (if these errors are expected)")?;
            writeln!(result, "- Add `#! error message` or `#! ^^^ error message` annotations")?;
            writeln!(result, "- Use `#! /regex/` or `#! ^^^ /regex/` for regex matching (e.g., `#! /could not find.*Baz/`)")?;
            writeln!(result, "- Annotation can be on the same line as the error OR on any following line")?;
            writeln!(result, "- The `^^^` markers indicate exact column positioning (optional)")?;
            writeln!(result, "- Without `^^^`, the diagnostic just needs to start somewhere on the most recent non-empty, non-comment line")?;
            writeln!(result, "- Look at other test files for annotation examples")?;
            writeln!(result)?;
            writeln!(result, "**Option 2: Fix the compiler/code** (if these errors are bugs)")?;
            writeln!(result, "- If diagnostics are incorrect, investigate the compiler logic")?;
            writeln!(result, "- If test code is wrong, fix the test source")?;
            writeln!(result)?;
            writeln!(result, "💡 **When in doubt**: Consult the user to clarify the test's intent")?;
            writeln!(result)?;
        }
        
        if auxiliary_failures > 0 {
            writeln!(result, "## Reference File Mismatch")?;
            writeln!(result)?;
            writeln!(result, "Output differs from reference files. If the new output is correct:")?;
            writeln!(result, "```bash")?;
            writeln!(result, "UPDATE_EXPECT=1 cargo dada test {}", self.path.to_string_lossy())?;
            writeln!(result, "```")?;
            writeln!(result)?;
        }
        
        if ice_failures > 0 {
            writeln!(result, "## Internal Compiler Error")?;
            writeln!(result)?;
            writeln!(result, "The compiler crashed - this indicates a compiler bug that needs investigation.")?;
            writeln!(result)?;
        }
        
        if spec_failures > 0 {
            writeln!(result, "## Invalid Spec Reference")?;
            writeln!(result)?;
            writeln!(result, "Fix the `#:spec` annotation to reference a valid spec paragraph.")?;
            writeln!(result)?;
        }
        
        if fixme_passed > 0 {
            writeln!(result, "## FIXME Test Passed")?;
            writeln!(result)?;
            writeln!(result, "This test was marked as FIXME but now passes - the bug may be fixed!")?;
            writeln!(result, "Consider removing the FIXME annotation.")?;
            writeln!(result)?;
        }
        
        Ok(())
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
