use std::path::PathBuf;

use crate::{
    ast::Identifier,
    span::{AbsoluteOffset, AbsoluteSpan, Anchor, Offset, Span, Spanned},
};

#[salsa::input]
pub struct CompilationRoot {
    #[return_ref]
    pub crates: Vec<CrateSource>,
}

impl CompilationRoot {
    pub fn crate_source<'db>(
        self,
        db: &'db dyn crate::Db,
        crate_name: Identifier<'db>,
    ) -> Option<CrateSource> {
        #[salsa::tracked]
        fn inner<'db>(
            db: &'db dyn crate::Db,
            root: CompilationRoot,
            crate_name: Identifier<'db>,
        ) -> Option<CrateSource> {
            let crate_name = crate_name.text(db);
            root.crates(db)
                .iter()
                .find(|c| c.name(db) == crate_name)
                .copied()
        }

        inner(db, self, crate_name)
    }

    pub fn libdada_crate<'db>(self, db: &'db dyn crate::Db) -> Option<CrateSource> {
        #[salsa::tracked]
        fn inner<'db>(db: &'db dyn crate::Db, root: CompilationRoot) -> Option<CrateSource> {
            root.crate_source(db, Identifier::dada(db))
        }

        inner(db, self)
    }
}

#[salsa::input]
pub struct CrateSource {
    #[return_ref]
    pub name: String,

    #[return_ref]
    pub kind: CrateKind,
}

#[derive(Debug)]
pub enum CrateKind {
    Directory(PathBuf),
}

#[salsa::input]
pub struct SourceFile {
    #[return_ref]
    pub path: String,

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
}
