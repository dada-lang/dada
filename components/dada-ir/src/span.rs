#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: Offset,
    pub end: Offset,
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

impl Span {
    #[track_caller]
    pub fn from(start: impl Into<Offset>, end: impl Into<Offset>) -> Self {
        Self {
            start: start.into(),
            end: end.into(),
        }
    }

    pub fn start() -> Self {
        Self {
            start: Offset(0),
            end: Offset(0),
        }
    }

    pub fn len(self) -> u32 {
        self.end - self.start
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
