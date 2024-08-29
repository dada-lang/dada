use std::path::{Path, PathBuf};

use dada_util::{bail, Fallible};
use rayon::prelude::*;
use walkdir::WalkDir;

use crate::{compiler::Compiler, TestOptions};

use super::Main;

struct FailedTest {
    path: String,
    details: String,
}

impl Main {
    pub(super) fn test(&mut self, options: &TestOptions) -> Fallible<()> {
        let tests = if options.inputs.is_empty() {
            self.assemble_tests(&["."])
        } else {
            self.assemble_tests(&options.inputs)
        };

        eprintln!("Total tests: {}", tests.len());

        let failed_tests: Vec<FailedTest> = tests
            .par_iter()
            .flat_map(|input| self.run_test(input))
            .collect();

        eprintln!("Failed tests: {}", failed_tests.len());

        for failed_test in &failed_tests {
            eprintln!("{}: {}", failed_test.path, failed_test.details);
        }

        if failed_tests.is_empty() {
            Ok(())
        } else {
            bail!("{} failed tests", failed_tests.len());
        }
    }

    fn assemble_tests(&self, inputs: &[impl AsRef<Path>]) -> Vec<PathBuf> {
        inputs
            .iter()
            .flat_map(|input| {
                let input: &Path = input.as_ref();
                if input.extension().map(|e| e == "dada").unwrap_or(false) {
                    vec![PathBuf::from(input)]
                } else {
                    WalkDir::new(input)
                        .into_iter()
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().extension().map(|e| e == "dada").unwrap_or(false))
                        .map(|e| e.into_path())
                        .collect::<Vec<_>>()
                }
            })
            .collect()
    }

    fn run_test(&self, input: &Path) -> Option<FailedTest> {
        let input: String = input.display().to_string();
        let mut compiler = Compiler::new();
        let source_file = match compiler.load_input(&input) {
            Ok(v) => v,
            Err(e) => {
                return Some(FailedTest {
                    path: input,
                    details: e.to_string(),
                })
            }
        };
        let diagnostics = compiler.parse(source_file);
        if !diagnostics.is_empty() {
            return Some(FailedTest {
                path: input,
                details: format!("{:?}", diagnostics),
            });
        }
        None
    }
}
