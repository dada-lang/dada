use std::path::{Path, PathBuf};

use dada_util::{bail, Fallible};
use rayon::prelude::*;
use walkdir::WalkDir;

use crate::{compiler::Compiler, TestOptions};

use super::Main;

pub(crate) struct FailedTest {
    path: String,
    details: String,
}

impl Main {
    pub fn test(&mut self, options: &TestOptions) -> Fallible<()> {
        let tests = self.assemble_tests(&options.inputs);

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

    fn assemble_tests(&self, inputs: &[String]) -> Vec<PathBuf> {
        inputs
            .iter()
            .flat_map(|input| {
                if input.ends_with(".dada") {
                    vec![PathBuf::from(input)]
                } else {
                    WalkDir::new(input)
                        .into_iter()
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().ends_with(".dada"))
                        .map(|e| e.into_path())
                        .collect::<Vec<_>>()
                }
            })
            .collect()
    }

    pub fn run_test(&self, input: &Path) -> Option<FailedTest> {
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
