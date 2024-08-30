use std::path::{Path, PathBuf};

use dada_ir_ast::diagnostic::Diagnostic;
use dada_util::{anyhow, bail, Fallible};
use indicatif::ProgressBar;
use rayon::prelude::*;
use walkdir::WalkDir;

use crate::{
    compiler::Compiler, db, error_reporting::RenderDiagnostic, GlobalOptions, TestOptions,
};

use super::Main;

#[derive(thiserror::Error, Debug)]
#[error("{} test failures", failed_tests.len())]
struct FailedTests {
    failed_tests: Vec<FailedTest>,
}

#[derive(Debug)]
struct FailedTest {
    path: PathBuf,
    unexpected_diagnostics: Vec<Diagnostic>,
    missing_diagnostics: Vec<Diagnostic>,
}

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

        return Ok(result);
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
        let source_file = compiler.load_input(&input_str)?;
        let diagnostics = compiler.parse(source_file);
        if diagnostics.is_empty() {
            return Ok(None);
        }

        let failed_test = FailedTest {
            path: input.to_path_buf(),
            unexpected_diagnostics: diagnostics,
            missing_diagnostics: vec![],
        };

        failed_test.generate_test_report(compiler.db())?;

        Ok(Some(failed_test))
    }
}

fn is_dada_file(input: &Path) -> bool {
    input.is_file() && input.extension().map(|e| e == "dada").unwrap_or(false)
}

impl FailedTest {
    fn summarize(&self) -> String {
        let mut result = vec![];
        if !self.unexpected_diagnostics.is_empty() {
            result.push(format!(
                "{} unexpected diagnostics",
                self.unexpected_diagnostics.len()
            ));
        }
        if !self.missing_diagnostics.is_empty() {
            result.push(format!(
                "{} missing diagnostics",
                self.missing_diagnostics.len()
            ));
        }
        result.push(format!("see {}", self.test_report_path().display()));
        result.join(", ")
    }

    fn test_report_path(&self) -> PathBuf {
        self.path.with_extension("test-report.md")
    }

    fn report(&self, db: &db::Database) -> Fallible<String> {
        use std::fmt::Write;
        let opts = GlobalOptions { no_color: true };

        let mut result = String::new();

        writeln!(result, "Test failed: {}", self.path.display())?;

        for diagnostic in &self.unexpected_diagnostics {
            writeln!(result, "")?;
            writeln!(result, "# Unexpected diagnostic")?;
            writeln!(result, "")?;

            let render = diagnostic.render(&opts, db);
            writeln!(result, "```\n{}\n```", render)?;
        }

        for diagnostic in &self.missing_diagnostics {
            writeln!(result, "")?;
            writeln!(result, "# Missing diagnostic")?;
            writeln!(result, "")?;

            let render = diagnostic.render(&opts, db);
            writeln!(result, "```\n{}\n```", render)?;
        }

        Ok(result)
    }

    fn generate_test_report(&self, db: &db::Database) -> Fallible<()> {
        std::fs::write(self.test_report_path(), self.report(db)?)?;
        Ok(())
    }
}
