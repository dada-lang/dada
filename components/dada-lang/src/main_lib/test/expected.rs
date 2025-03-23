use std::path::{Path, PathBuf};

use dada_compiler::Compiler;
use dada_ir_ast::{
    diagnostic::Diagnostic,
    inputs::SourceFile,
    span::{AbsoluteOffset, AbsoluteSpan},
};
use dada_util::{Context, Fallible, bail};
use prettydiff::text::ContextConfig;
use regex::Regex;

use crate::GlobalOptions;

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
    bless: Bless,
    expected_diagnostics: Vec<ExpectedDiagnostic>,
    fn_asts: bool,
}

enum Bless {
    None,
    All,
    File(String),
}

lazy_static::lazy_static! {
    static ref UNINTERESTING_RE: Regex = Regex::new(r"^\s*(#.*)?$").unwrap();
}

lazy_static::lazy_static! {
    static ref DIAGNOSTIC_RE: Regex = Regex::new(r"^(?P<pre>[^#]*)#!(?P<pad>\s*)(?P<col>\^+)?(?P<re> /)?\s*(?P<msg>.*)").unwrap();
}

impl TestExpectations {
    pub fn new(db: &dyn crate::Db, source_file: SourceFile) -> Fallible<Self> {
        let bless = match std::env::var("UPDATE_EXPECT") {
            Ok(s) => {
                if s == "1" {
                    Bless::All
                } else {
                    Bless::File(s)
                }
            }
            Err(_) => Bless::None,
        };

        let mut expectations = TestExpectations {
            source_file,
            bless,
            expected_diagnostics: vec![],
            fn_asts: false,
        };
        expectations.initialize(db)?;
        Ok(expectations)
    }

    fn initialize(&mut self, db: &dyn crate::Db) -> Fallible<()> {
        let source = self.source_file.contents_if_ok(db);
        let line_starts = std::iter::once(0)
            .chain(
                source
                    .char_indices()
                    .filter_map(|(i, c)| (c == '\n').then_some(i + 1)),
            )
            .chain(std::iter::once(source.len()))
            .collect::<Vec<_>>();

        let mut in_header = true;
        let mut last_interesting_line = None;
        for (line, line_index) in source.lines().zip(0..) {
            // Allow `#:` configuration lines, but only at the start of the file.
            if in_header {
                if let Some(suffix) = line.strip_prefix("#:") {
                    self.configuration(db, line_index, suffix.trim())?;
                    continue;
                } else if line.starts_with("#") || line.trim().is_empty() {
                    continue;
                }
            }

            // Otherwise error if we see `#:`.
            in_header = false;
            if line.contains("#:") {
                bail!(
                    "{}:{}: configuration comment outside of file header",
                    self.source_file.url_display(db),
                    line_index + 1,
                );
            }

            // Track the last "interesting" line (non-empty, basically).
            // Any future `#!` errors will be assumed to start on that line.
            if !UNINTERESTING_RE.is_match(line) {
                last_interesting_line = Some(line_index);
            }

            // Check if this line contains an expected diagnostic.
            let Some(c) = DIAGNOSTIC_RE.captures(line) else {
                continue;
            };

            // Find the line on which the diagnostic will be expected to occur.
            let Some(last_interesting_line) = last_interesting_line else {
                bail!("found diagnostic on line with no previous interesting line");
            };

            // Extract the expected span: if the comment contains `^^^` markers, it needs to be
            // exactly as given, but otherwise it just has to start somewhere on the line.
            let pre = c.name("pre").unwrap().as_str();
            let pad = c.name("pad").unwrap().as_str();
            let span = match c.name("col") {
                Some(c) => {
                    let carrot_start =
                        line_starts[last_interesting_line] + pre.len() + 2 + pad.len();
                    let carrot_end = carrot_start + c.as_str().len();

                    ExpectedSpan::MustEqual(AbsoluteSpan {
                        source_file: self.source_file,
                        start: AbsoluteOffset::from(carrot_start),
                        end: AbsoluteOffset::from(carrot_end),
                    })
                }
                None => ExpectedSpan::MustStartWithin(AbsoluteSpan {
                    source_file: self.source_file,
                    start: AbsoluteOffset::from(line_starts[last_interesting_line]),
                    end: AbsoluteOffset::from(
                        line_starts[last_interesting_line + 1].saturating_sub(1),
                    ),
                }),
            };

            // Find the expected message (which may be a regular expression).
            let message = match c.name("re") {
                Some(_) => Regex::new(c.name("msg").unwrap().as_str())?,
                None => Regex::new(&regex::escape(c.name("msg").unwrap().as_str()))?,
            };

            // Push onto the list of expected diagnostics.
            self.expected_diagnostics
                .push(ExpectedDiagnostic { span, message });
        }

        self.expected_diagnostics.sort_by_key(|e| *e.span());

        Ok(())
    }

    fn configuration(&mut self, db: &dyn crate::Db, line_index: usize, line: &str) -> Fallible<()> {
        if line == "fn_asts" {
            self.fn_asts = true;
            return Ok(());
        }

        bail!(
            "{}:{}: unrecognized configuration comment",
            self.source_file.url_display(db),
            line_index + 1,
        );
    }

    pub fn compare(self, compiler: &mut Compiler) -> Fallible<Option<FailedTest>> {
        use std::fmt::Write;

        let mut test = FailedTest {
            path: self.source_file.url(compiler).to_file_path().unwrap(),
            full_compiler_output: Default::default(),
            failures: vec![],
        };

        test.failures.extend(self.compare_auxiliary(
            compiler,
            "fn_asts",
            self.fn_asts,
            Self::generate_fn_asts,
        )?);

        let actual_diagnostics = compiler.check_all(self.source_file);
        let _wasm_bytes = compiler.codegen_main_fn(self.source_file);

        for diagnostic in &actual_diagnostics {
            writeln!(
                test.full_compiler_output,
                "{}",
                diagnostic.render(compiler, &GlobalOptions::test_options().render_opts())
            )?;
        }

        test.failures
            .extend(self.compare_diagnostics(actual_diagnostics));

        if test.failures.is_empty() {
            Ok(None)
        } else {
            Ok(Some(test))
        }
    }

    fn generate_fn_asts(&self, compiler: &mut Compiler) -> String {
        compiler.fn_asts(self.source_file)
    }

    fn compare_auxiliary(
        &self,
        compiler: &mut Compiler,
        ext: &str,
        enabled: bool,
        generate_fn: impl Fn(&Self, &mut Compiler) -> String,
    ) -> Fallible<Vec<Failure>> {
        let ref_path = self.ref_path(compiler, ext);
        let txt_path = self.txt_path(compiler, ext);

        if !enabled {
            self.remove_stale_file(&ref_path)?;
            self.remove_stale_file(&txt_path)?;
            return Ok(vec![]);
        }

        let actual = generate_fn(self, compiler);
        self.write_file(&txt_path, &actual)?;

        if self.bless.bless_path(&ref_path) {
            self.write_file(&ref_path, &actual)?;
            return Ok(vec![]);
        }

        let expected = std::fs::read_to_string(&ref_path).unwrap_or_default();
        if actual == expected {
            return Ok(vec![]);
        }

        let diff = self.diff_lines(&expected, &actual);
        Ok(vec![Failure::Auxiliary {
            kind: format!(":{ext}"),
            ref_path,
            txt_path,
            diff,
        }])
    }

    fn remove_stale_file(&self, path: &Path) -> Fallible<()> {
        if path.exists() {
            std::fs::remove_file(path)
                .with_context(|| format!("removing stale file `{}`", path.display()))?;
        }

        Ok(())
    }

    fn write_file(&self, path: &Path, contents: &str) -> Fallible<()> {
        std::fs::write(path, contents)
            .with_context(|| format!("writing to file `{}`", path.display()))?;
        Ok(())
    }

    fn compare_diagnostics(self, mut actual_diagnostics: Vec<&Diagnostic>) -> Vec<Failure> {
        actual_diagnostics.sort_by_key(|d| d.span);

        let empty_matched = vec![false; self.expected_diagnostics.len()];
        let mut matched = empty_matched.clone();

        // Make sure that every actual diagnostic matches some expected diagnostic
        let mut failures = vec![];

        for actual_diagnostic in actual_diagnostics {
            // Check whether this matches an expected diagnostic that
            // has not yet been matched.
            if let Some(index) = self.find_match(actual_diagnostic, &matched) {
                matched[index] = true; // Good!
                continue;
            }

            // Check whether this matches an expected diagnostic that
            // had already matched.
            match self.find_match(actual_diagnostic, &empty_matched) {
                Some(index) => {
                    failures.push(Failure::MultipleMatches(
                        self.expected_diagnostics[index].clone(),
                        actual_diagnostic.clone(),
                    ));
                }
                None => {
                    failures.push(Failure::UnexpectedDiagnostic(actual_diagnostic.clone()));
                }
            }
        }

        for (expected_diagnostic, matched) in self.expected_diagnostics.into_iter().zip(matched) {
            if !matched {
                failures.push(Failure::MissingDiagnostic(expected_diagnostic));
            }
        }

        failures
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
            // Find the best match (with the narrowest span)
            .min_by_key(|(expected_diagnostic, _)| expected_diagnostic.span())
            .map(|(_, index)| index)
    }

    pub fn source_path(&self, db: &dyn crate::Db) -> PathBuf {
        self.source_file.url(db).to_file_path().unwrap()
    }

    fn ref_path(&self, db: &dyn crate::Db, ext: &str) -> PathBuf {
        let path_buf = self.source_path(db);
        path_buf.with_extension(format!("{ext}.ref"))
    }

    fn txt_path(&self, db: &dyn crate::Db, ext: &str) -> PathBuf {
        let path_buf = self.source_path(db);
        path_buf.with_extension(format!("{ext}.txt"))
    }

    fn diff_lines(&self, expected: &str, actual: &str) -> String {
        prettydiff::diff_lines(expected, actual)
            .set_diff_only(true)
            .format_with_context(
                Some(ContextConfig {
                    context_size: 3,
                    skipping_marker: "...",
                }),
                true,
            )
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

impl Bless {
    fn bless_path(&self, path: &Path) -> bool {
        match self {
            Bless::None => false,
            Bless::All => true,
            Bless::File(s) => path.file_name().unwrap() == &s[..],
        }
    }
}
