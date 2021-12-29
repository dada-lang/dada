use crate::filename::Filename;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FileSpan {
    pub filename: Filename,
    pub start: Offset,
    pub end: Offset,
}

impl<'db> salsa::DebugWithDb<'db> for FileSpan {
    type Db = dyn crate::Db + 'db;
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        write!(
            f,
            "{}@{:?}",
            self.filename.as_str(db),
            Span {
                start: self.start,
                end: self.end
            }
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
/// 0-based byte offset with=in a file.
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

    /// Returns a 0-length span at the start of this span
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

impl Into<u32> for Offset {
    fn into(self) -> u32 {
        self.0
    }
}

impl Into<usize> for Offset {
    fn into(self) -> usize {
        self.0 as usize
    }
}
