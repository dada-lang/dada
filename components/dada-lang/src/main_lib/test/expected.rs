use std::path::PathBuf;

use dada_ir_ast::{
    diagnostic::Diagnostic,
    inputs::SourceFile,
    span::{AbsoluteOffset, AbsoluteSpan},
};
use dada_util::Fallible;
use regex::Regex;

use crate::db;

use super::{FailedTest, Failure};

#[derive(Clone, Debug)]
pub struct ExpectedDiagnostic {
    /// The span where this diagnostic is expected to start.
    /// The start of some actual diagnostic must fall within this span.
    span: ExpectedSpan,

    /// regular expression that message must match
    message: Regex,
}

#[derive(Copy, Clone, Debug)]
pub enum ExpectedSpan {
    MustStartWithin(AbsoluteSpan),
    MustEqual(AbsoluteSpan),
}

pub struct TestExpectations {
    source_file: SourceFile,
    expected_diagnostics: Vec<ExpectedDiagnostic>,
}

lazy_static::lazy_static! {
    static ref UNINTERESTING_RE: Regex = Regex::new(r"^\s*(#.*)?$").unwrap();
}

lazy_static::lazy_static! {
    static ref DIAGNOSTIC_RE: Regex = Regex::new(r"^(?P<pre>[^#]*)#!(?P<pad>\s*)(?P<col>\^+)?(?P<re> /)?\s*(?P<msg>.*)").unwrap();
}

impl TestExpectations {
    pub fn new(db: &db::Database, source_file: SourceFile) -> Fallible<Self> {
        let source = source_file.contents(db);

        let line_starts = std::iter::once(0)
            .chain(
                source
                    .char_indices()
                    .filter_map(|(i, c)| (c == '\n').then_some(i + 1)),
            )
            .chain(std::iter::once(source.len()))
            .collect::<Vec<_>>();

        let mut expected_diagnostics = vec![];
        let mut last_interesting_line = None;
        for (line, line_index) in source.lines().zip(0..) {
            if !UNINTERESTING_RE.is_match(line) {
                last_interesting_line = Some(line_index);
            }

            let Some(c) = DIAGNOSTIC_RE.captures(line) else {
                continue;
            };

            let Some(last_interesting_line) = last_interesting_line else {
                dada_util::bail!("found diagnostic on line with no previous interesting line");
            };

            let pre = c.name("pre").unwrap().as_str();
            let pad = c.name("pad").unwrap().as_str();

            let span = match c.name("col") {
                Some(c) => {
                    let carrot_start =
                        line_starts[last_interesting_line] + pre.len() + 2 + pad.len();
                    let carrot_end = carrot_start + c.as_str().len();

                    ExpectedSpan::MustEqual(AbsoluteSpan {
                        source_file,
                        start: AbsoluteOffset::from(carrot_start),
                        end: AbsoluteOffset::from(carrot_end),
                    })
                }
                None => ExpectedSpan::MustStartWithin(AbsoluteSpan {
                    source_file,
                    start: AbsoluteOffset::from(line_starts[last_interesting_line]),
                    end: AbsoluteOffset::from(
                        line_starts[last_interesting_line + 1].saturating_sub(1),
                    ),
                }),
            };

            let message = match c.name("re") {
                Some(_) => Regex::new(c.name("msg").unwrap().as_str())?,
                None => Regex::new(&regex::escape(c.name("msg").unwrap().as_str()))?,
            };

            expected_diagnostics.push(ExpectedDiagnostic { span, message });
        }

        expected_diagnostics.sort_by_key(|e| *e.span());

        Ok(TestExpectations {
            source_file,
            expected_diagnostics,
        })
    }

    pub fn compare(
        self,
        db: &db::Database,
        mut actual_diagnostics: Vec<Diagnostic>,
    ) -> Option<FailedTest> {
        actual_diagnostics.sort_by_key(|d| d.span);

        let empty_matched = vec![false; self.expected_diagnostics.len()];
        let mut matched = empty_matched.clone();

        // Make sure that every actual diagnostic matches some expected diagnostic
        let mut failures = vec![];

        for actual_diagnostic in actual_diagnostics {
            // Check whether this matches an expected diagnostic that
            // has not yet been matched.
            if let Some(index) = self.find_match(&actual_diagnostic, &matched) {
                matched[index] = true; // Good!
                continue;
            }

            // Check whether this matches an expected diagnostic that
            // had already matched.
            match self.find_match(&actual_diagnostic, &empty_matched) {
                Some(index) => {
                    failures.push(Failure::MultipleMatches(
                        self.expected_diagnostics[index].clone(),
                        actual_diagnostic,
                    ));
                }
                None => {
                    failures.push(Failure::UnexpectedDiagnostic(actual_diagnostic));
                }
            }
        }

        for (expected_diagnostic, matched) in self.expected_diagnostics.into_iter().zip(matched) {
            if !matched {
                failures.push(Failure::MissingDiagnostic(expected_diagnostic));
            }
        }

        if failures.is_empty() {
            return None;
        }

        Some(FailedTest {
            path: PathBuf::from(self.source_file.path(db)),
            failures,
        })
    }

    fn find_match(&self, actual_diagnostic: &Diagnostic, matched: &[bool]) -> Option<usize> {
        self.expected_diagnostics
            .iter()
            .zip(0_usize..)
            .filter(|&(expected_diagnostic, index)| {
                !matched[index]
                    && expected_diagnostic.span.matches(&actual_diagnostic.span)
                    && expected_diagnostic
                        .message
                        .is_match(&actual_diagnostic.message)
            })
            .min_by_key(|(expected_diagnostic, _)| expected_diagnostic.span())
            .map(|(_, index)| index)
    }
}

impl ExpectedDiagnostic {
    pub fn span(&self) -> &AbsoluteSpan {
        self.span.span()
    }
}

impl ExpectedSpan {
    pub fn matches(&self, actual_span: &AbsoluteSpan) -> bool {
        match self {
            ExpectedSpan::MustStartWithin(expected_span) => {
                expected_span.start <= actual_span.start && actual_span.start <= expected_span.end
            }
            ExpectedSpan::MustEqual(expected_span) => expected_span == actual_span,
        }
    }

    pub fn span(&self) -> &AbsoluteSpan {
        match self {
            ExpectedSpan::MustStartWithin(span) => span,
            ExpectedSpan::MustEqual(span) => span,
        }
    }
}
