use std::path::{Path, PathBuf};

use dada_ir_ast::diagnostic::Diagnostic;
use dada_util::{anyhow, bail, Fallible};
use expected::ExpectedDiagnostic;
use indicatif::ProgressBar;
use rayon::prelude::*;
use walkdir::WalkDir;

use crate::{
    compiler::Compiler, db, error_reporting::RenderDiagnostic, GlobalOptions, TestOptions,
};

use super::Main;

mod expected;

#[derive(thiserror::Error, Debug)]
#[error("{} test failures", failed_tests.len())]
struct FailedTests {
    failed_tests: Vec<FailedTest>,
}

#[derive(Debug)]
struct FailedTest {
    path: PathBuf,
    failures: Vec<Failure>,
}

#[derive(Debug)]
enum Failure {
    UnexpectedDiagnostic(Diagnostic),
    MultipleMatches(ExpectedDiagnostic, Diagnostic),
    MissingDiagnostic(ExpectedDiagnostic),

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

impl Main {
    pub(super) fn test(&mut self, options: &TestOptions) -> Fallible<()> {
        let tests = if options.inputs.is_empty() {
            self.assemble_tests(&["."])?
        } else {
            self.assemble_tests(&options.inputs)?
        };

        let progress_bar = ProgressBar::new(tests.len() as u64);

        let failed_or_errored_tests: Vec<Fallible<Option<FailedTest>>> = tests
            .par_iter()
            .map(|input| {
                let result = self.run_test(input);
                match &result {
                    Ok(None) => {}
                    Ok(Some(error)) => {
                        progress_bar.println(format!("{}: {}", input.display(), error.summarize()))
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
            .collect();

        let mut failed_tests = vec![];
        for failed_or_errored_test in failed_or_errored_tests {
            failed_tests.extend(failed_or_errored_test?);
        }

        if failed_tests.is_empty() {
            progress_bar.finish_with_message(format!("All {} tests passed", tests.len()));

            Ok(())
        } else {
            Err(FailedTests { failed_tests }.into())
        }
    }

    fn assemble_tests(&self, inputs: &[impl AsRef<Path>]) -> Fallible<Vec<PathBuf>> {
        let mut result = vec![];

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
    /// * `Ok(Some(error))` if the test failed.
    /// * `Ok(None)` if the test passed.
    fn run_test(&self, input: &Path) -> Fallible<Option<FailedTest>> {
        assert!(is_dada_file(input));
        let input_str = input
            .as_os_str()
            .to_str()
            .ok_or_else(|| anyhow!("path cannot be represented in utf-8: `{:?}`", input))?;
        let mut compiler = Compiler::new();
        let source_file = compiler.load_input(input_str)?;
        let expectations = expected::TestExpectations::new(compiler.db(), source_file)?;
        match expectations.compare(&mut compiler)? {
            None => {
                delete_test_report(input)?;
                Ok(None)
            }
            Some(failed_test) => {
                failed_test.generate_test_report(compiler.db())?;
                Ok(Some(failed_test))
            }
        }
    }
}

fn is_dada_file(input: &Path) -> bool {
    input.is_file() && input.extension().map(|e| e == "dada").unwrap_or(false)
}

impl FailedTest {
    fn summarize(&self) -> String {
        format!(
            "{} failures, see {}",
            self.failures.len(),
            test_report_path(&self.path).display()
        )
    }

    fn report(&self, db: &db::Database) -> Fallible<String> {
        use std::fmt::Write;
        let opts = GlobalOptions { no_color: true };

        let mut result = String::new();

        writeln!(result, "Test failed: {}", self.path.display())?;

        for failure in &self.failures {
            match failure {
                Failure::UnexpectedDiagnostic(diagnostic) => {
                    writeln!(result)?;
                    writeln!(result, "# Unexpected diagnostic")?;
                    writeln!(result)?;

                    let render = diagnostic.render(&opts, db);
                    writeln!(result, "```\n{}\n```", render)?;
                    writeln!(result)?;
                    writeln!(result, "```\n{diagnostic:#?}\n```\n")?;
                }
                Failure::MultipleMatches(expected, actual) => {
                    writeln!(result)?;
                    writeln!(result, "# Multiple matches for expected diagnostic")?;
                    writeln!(result)?;

                    writeln!(result, "Diagnostic:")?;
                    let render = actual.render(&opts, db);
                    writeln!(result, "```\n{}\n```", render)?;
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

                    writeln!(result, "```\n{expected:#?}\n```")?;
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

    fn generate_test_report(&self, db: &db::Database) -> Fallible<()> {
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
