use crate::filename::Filename;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FileSpan {
    pub filename: Filename,
    pub start: Offset,
    pub end: Offset,
}

impl FileSpan {
    pub fn snippet<'db>(&self, db: &'db dyn crate::Db) -> &'db str {
        &crate::manifest::source_text(db, self.filename)
            [usize::from(self.start)..usize::from(self.end)]
    }

    /// True if the given character falls within this span.
    pub fn contains(&self, offset: Offset) -> bool {
        self.start <= offset && offset < self.end
    }
}

impl<Db: ?Sized + crate::Db> salsa::DebugWithDb<Db> for FileSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        let db = db.as_dyn_ir_db();
        let start = crate::lines::line_column(db, self.filename, self.start);
        let end = crate::lines::line_column(db, self.filename, self.end);
        write!(
            f,
            "{}:{}:{}:{}:{}",
            self.filename.as_str(db),
            start.line,
            start.column,
            end.line,
            end.column,
        )
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: Offset,
    pub end: Offset,
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?}..{:?})", self.start.0, self.end.0)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// 0-based byte offset within a file.
pub struct Offset(u32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct LineColumn {
    /// 1-based line number
    pub line: u32,

    /// 1-based column nuimber
    pub column: u32,
}

impl From<FileSpan> for Span {
    fn from(fs: FileSpan) -> Span {
        Span {
            start: fs.start,
            end: fs.end,
        }
    }
}

impl Span {
    #[track_caller]
    pub fn from(start: impl Into<Offset>, end: impl Into<Offset>) -> Self {
        let this = Self {
            start: start.into(),
            end: end.into(),
        };
        assert!(this.start <= this.end);
        this
    }

    pub fn in_file(self, filename: Filename) -> FileSpan {
        FileSpan {
            filename,
            start: self.start,
            end: self.end,
        }
    }

    pub fn snippet<'db>(&self, db: &'db dyn crate::Db, filename: Filename) -> &'db str {
        self.in_file(filename).snippet(db)
    }

    /// Returns a 0-length span at the start of this span
    #[must_use]
    pub fn span_at_start(self) -> Span {
        Span {
            start: self.start,
            end: self.start,
        }
    }

    pub fn zero() -> Self {
        Self {
            start: Offset(0),
            end: Offset(0),
        }
    }

    pub fn len(self) -> u32 {
        self.end - self.start
    }

    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub fn to(self, other: Span) -> Span {
        assert!(self == other || self.end <= other.start);
        Span {
            start: self.start,
            end: other.end,
        }
    }
}

impl std::ops::Add<u32> for Offset {
    type Output = Offset;

    fn add(self, other: u32) -> Offset {
        Offset(self.0 + other)
    }
}

impl std::ops::Add<usize> for Offset {
    type Output = Offset;

    fn add(self, other: usize) -> Offset {
        assert!(other < std::u32::MAX as usize);
        self + (other as u32)
    }
}

impl std::ops::Sub<Offset> for Offset {
    type Output = u32;

    fn sub(self, other: Offset) -> u32 {
        self.0 - other.0
    }
}

impl From<usize> for Offset {
    fn from(value: usize) -> Offset {
        assert!(value < std::u32::MAX as usize);
        Offset(value as u32)
    }
}

impl From<u32> for Offset {
    fn from(value: u32) -> Offset {
        Offset(value)
    }
}

impl From<Offset> for u32 {
    fn from(offset: Offset) -> Self {
        offset.0
    }
}

impl From<Offset> for usize {
    fn from(offset: Offset) -> Self {
        offset.0 as usize
    }
}
