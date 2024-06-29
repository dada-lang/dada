use salsa::{DebugWithDb, Update};

use crate::{ast::Item, inputs::SourceFile};

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, DebugWithDb, Debug, Update)]
pub struct Span<'db> {
    pub anchor: Item<'db>,
    pub start: Offset,
    pub end: Offset,
}

impl<'db> Span<'db> {
    pub fn to(self, end: Span) -> Span<'db> {
        assert!(self.anchor == end.anchor);
        Span {
            anchor: self.anchor,
            start: self.start,
            end: end.end,
        }
    }

    pub fn absolute(&self, db: &'db dyn crate::Db) -> (SourceFile, AbsoluteOffset, AbsoluteOffset) {
        let (source_file, anchor_start) = self.anchor.absolute_start(db);
        (
            source_file,
            anchor_start + self.start,
            anchor_start + self.end,
        )
    }

    pub fn absolute_start(&self, db: &'db dyn crate::Db) -> (SourceFile, AbsoluteOffset) {
        let (source_file, offset) = self.anchor.absolute_start(db);
        (source_file, offset + self.start)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, DebugWithDb, Debug)]
pub struct Offset(u32);

impl From<usize> for Offset {
    fn from(offset: usize) -> Self {
        assert!(offset < std::u32::MAX as usize);
        Offset(offset as u32)
    }
}

impl From<u32> for Offset {
    fn from(offset: u32) -> Self {
        Offset(offset)
    }
}

impl Offset {
    pub const ZERO: Offset = Offset(0);

    pub fn as_u32(&self) -> u32 {
        self.0
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl std::ops::Add<usize> for Offset {
    type Output = Offset;

    fn add(self, rhs: usize) -> Self::Output {
        assert!(rhs < std::u32::MAX as usize);
        Offset(self.0.checked_add(rhs as u32).unwrap())
    }
}

impl std::ops::Add<Offset> for Offset {
    type Output = Offset;

    fn add(self, rhs: Offset) -> Self::Output {
        Offset(self.0.checked_add(rhs.0).unwrap())
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, DebugWithDb, Debug)]
pub struct AbsoluteOffset(u32);

impl AbsoluteOffset {
    pub const ZERO: AbsoluteOffset = AbsoluteOffset(0);
}

impl std::ops::Add<Offset> for AbsoluteOffset {
    type Output = AbsoluteOffset;

    fn add(self, rhs: Offset) -> Self::Output {
        AbsoluteOffset(self.0.checked_add(rhs.0).unwrap())
    }
}
