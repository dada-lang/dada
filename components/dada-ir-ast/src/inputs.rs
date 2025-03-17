use std::ops::Range;

use dada_util::SalsaSerialize;
use url::Url;

use crate::{
    ast::Identifier,
    span::{AbsoluteOffset, AbsoluteSpan, Anchor, Offset, Span, Spanned, ZeroColumn, ZeroLine},
};

#[derive(SalsaSerialize)]
#[salsa::input]
pub struct CompilationRoot {
    #[return_ref]
    pub crates: Vec<Krate>,
}

impl CompilationRoot {
    pub fn crate_source<'db>(
        self,
        db: &'db dyn crate::Db,
        crate_name: Identifier<'db>,
    ) -> Option<Krate> {
        #[salsa::tracked]
        fn inner<'db>(
            db: &'db dyn crate::Db,
            root: CompilationRoot,
            crate_name: Identifier<'db>,
        ) -> Option<Krate> {
            let crate_name = crate_name.text(db);
            root.crates(db)
                .iter()
                .find(|c| c.name(db) == crate_name)
                .copied()
        }

        inner(db, self, crate_name)
    }

    /// Returns the [`Krate`][] for the `libdada` crate.
    /// The creator of the [`CompilationRoot`][] is responsible for ensuring that this crate is present.
    pub fn libdada_crate(self, db: &dyn crate::Db) -> Krate {
        #[salsa::tracked]
        fn inner<'db>(db: &'db dyn crate::Db, root: CompilationRoot) -> Krate {
            root.crate_source(db, Identifier::dada(db))
                .expect("libdada crate not found")
        }

        inner(db, self)
    }
}

#[derive(SalsaSerialize)]
#[salsa::input]
pub struct Krate {
    #[return_ref]
    pub name: String,
}

#[derive(SalsaSerialize)]
#[salsa::input]
pub struct SourceFile {
    #[return_ref]
    pub url: Url,

    /// Contents of the source file or an error message if it was not possible to read it.
    #[return_ref]
    pub contents: Result<String, String>,
}

impl<'db> Spanned<'db> for SourceFile {
    fn span(&self, db: &'db dyn crate::Db) -> crate::span::Span<'db> {
        Span {
            anchor: Anchor::SourceFile(*self),
            start: Offset::ZERO,
            end: Offset::from(self.contents_if_ok(db).len()),
        }
    }
}

#[salsa::tracked]
impl SourceFile {
    /// Returns the contents of this file or an empty string if it couldn't be read.
    pub fn contents_if_ok(self, db: &dyn crate::Db) -> &str {
        match self.contents(db) {
            Ok(s) => s,
            Err(_) => "",
        }
    }

    /// Returns an absolute span representing the entire source file.
    pub fn absolute_span(self, db: &dyn crate::Db) -> AbsoluteSpan {
        AbsoluteSpan {
            source_file: self,
            start: AbsoluteOffset::ZERO,
            end: AbsoluteOffset::from(self.contents_if_ok(db).len()),
        }
    }

    pub fn module_name(self, db: &dyn crate::Db) -> Identifier<'_> {
        let url = self.url(db);
        let module_name = url
            .path_segments()
            .and_then(|mut segments| segments.next_back())
            .unwrap_or("<input>");
        Identifier::new(db, module_name)
    }

    pub fn url_display(self, db: &dyn crate::Db) -> String {
        db.url_display(self.url(db))
    }

    /// A vector containing the start indices of each (0-based) line
    /// plus one final entry with the total document length
    /// (effectively an imaginary N+1 line that starts, and ends, at the end).
    #[salsa::tracked(return_ref)]
    pub fn line_starts(self, db: &dyn crate::Db) -> Vec<AbsoluteOffset> {
        std::iter::once(0)
            .chain(
                self.contents_if_ok(db)
                    .char_indices()
                    .filter(|&(_, ch)| ch == '\n')
                    .map(|(index, _)| index + 1),
            )
            .chain(std::iter::once(self.contents_if_ok(db).len()))
            .map(AbsoluteOffset::from)
            .collect()
    }

    pub fn line_range(self, db: &dyn crate::Db, line: ZeroLine) -> Range<AbsoluteOffset> {
        let line_starts = self.line_starts(db);
        line_starts[line.as_usize()]..line_starts[line.as_usize() + 1]
    }

    pub fn line_col(self, db: &dyn crate::Db, offset: AbsoluteOffset) -> (ZeroLine, ZeroColumn) {
        let line_starts = self.line_starts(db);
        match line_starts.iter().position(|&s| s > offset) {
            Some(next_line) => {
                assert!(next_line > 0);
                let line_index = next_line - 1;
                let line_start = line_starts[line_index];
                (
                    ZeroLine::from(line_index),
                    ZeroColumn::from(offset - line_start),
                )
            }
            None => {
                // This must be the end of the document. We will return the last column on the last line.
                let last_line = self.line_starts(db).len() - 1;
                let last_line_start = line_starts[last_line];
                (
                    ZeroLine::from(last_line),
                    ZeroColumn::from(offset - last_line_start),
                )
            }
        }
    }
}
