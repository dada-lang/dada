use salsa::Update;

use crate::{ast::Item, inputs::SourceFile};

/// A span within the input.
///
/// The offsets are stored relative to the start of the **anchor**,
/// which is some item (e.g., a class, function, etc). The use of relative offsets avoids
/// incremental churn if lines or content is added before/after the definition.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Update)]
pub struct Span<'db> {
    pub anchor: Item<'db>,
    pub start: Offset,
    pub end: Offset,
}

/// An absolute span within the input. The offsets are stored as absolute offsets
/// within a given source file. These are used for diagnostics or outputs but not
/// internally during compilation.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct AbsoluteSpan {
    pub source_file: SourceFile,
    pub start: AbsoluteOffset,
    pub end: AbsoluteOffset,
}

impl<'db> Span<'db> {
    pub fn to(self, end: impl IntoOptionSpan<'db>) -> Span<'db> {
        match end.into_opt_span() {
            Some(end) => {
                assert!(self.anchor == end.anchor);
                Span {
                    anchor: self.anchor,
                    start: self.start,
                    end: end.end,
                }
            }
            None => self,
        }
    }

    pub fn start_from(self, start: impl IntoOptionSpan<'db>) -> Span<'db> {
        match start.into_opt_span() {
            Some(start) => {
                assert!(self.anchor == start.anchor);
                Span {
                    anchor: self.anchor,
                    start: start.start,
                    end: self.end,
                }
            }
            None => self,
        }
    }

    /// Span pointing at the start of `self`
    pub fn at_start(self) -> Span<'db> {
        Span {
            anchor: self.anchor,
            start: self.end,
            end: self.end,
        }
    }

    /// Span pointing at the end of `self`
    pub fn at_end(self) -> Span<'db> {
        Span {
            anchor: self.anchor,
            start: self.end,
            end: self.end,
        }
    }

    /// Convert this span into an absolute span for reporting errors.
    pub fn absolute_span(&self, db: &'db dyn crate::Db) -> AbsoluteSpan {
        let anchor_span = self.anchor.absolute_span(db);
        AbsoluteSpan {
            source_file: anchor_span.source_file,
            start: anchor_span.start + self.start,
            end: anchor_span.start + self.end,
        }
    }
}

impl<'db> Spanned<'db> for Span<'db> {
    fn span(&self, _db: &'db dyn crate::Db) -> Span<'db> {
        *self
    }
}

/// Implemented by all things that have a span (and span itself)
pub trait Spanned<'db> {
    fn span(&self, db: &'db dyn crate::Db) -> Span<'db>;
}

/// Either `Span` or `Option<Span>`.
pub trait IntoOptionSpan<'db> {
    fn into_opt_span(self) -> Option<Span<'db>>;
}

impl<'db> IntoOptionSpan<'db> for Span<'db> {
    fn into_opt_span(self) -> Option<Span<'db>> {
        Some(self)
    }
}

impl<'db> IntoOptionSpan<'db> for Option<Span<'db>> {
    fn into_opt_span(self) -> Option<Span<'db>> {
        self
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Offset(u32);

impl From<usize> for Offset {
    fn from(offset: usize) -> Self {
        assert!(offset < u32::MAX as usize);
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
        assert!(rhs < u32::MAX as usize);
        Offset(self.0.checked_add(rhs as u32).unwrap())
    }
}

impl std::ops::Add<Offset> for Offset {
    type Output = Offset;

    fn add(self, rhs: Offset) -> Self::Output {
        Offset(self.0.checked_add(rhs.0).unwrap())
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct AbsoluteOffset(u32);

impl AbsoluteOffset {
    pub const ZERO: AbsoluteOffset = AbsoluteOffset(0);

    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl From<usize> for AbsoluteOffset {
    fn from(offset: usize) -> Self {
        assert!(offset < u32::MAX as usize);
        AbsoluteOffset(offset as u32)
    }
}

impl From<u32> for AbsoluteOffset {
    fn from(offset: u32) -> Self {
        AbsoluteOffset(offset)
    }
}

impl std::ops::Add<Offset> for AbsoluteOffset {
    type Output = AbsoluteOffset;

    fn add(self, rhs: Offset) -> Self::Output {
        AbsoluteOffset(self.0.checked_add(rhs.0).unwrap())
    }
}

impl PartialOrd for AbsoluteSpan {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(Self::cmp(self, other))
    }
}

/// Span A < Span B if:
///
/// * A is enclosed in B
/// * A ends before B ends
/// * A starts before B starts
impl Ord for AbsoluteSpan {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let AbsoluteSpan {
            source_file,
            start,
            end,
        } = self;
        let AbsoluteSpan {
            source_file: other_source_file,
            start: other_start,
            end: other_end,
        } = other;

        // Same starting point...

        //      ^^^^^^^^^^^^^
        //           ==
        //      ^^^^^^^^^^^^^

        //      ^^^^^^^^^^^^^^^^
        //           >
        //      ^^^^^^^^^^^^^

        //      ^^^^^^^^
        //           <
        //      ^^^^^^^^^^^^^

        // Less starting point...

        //      ^^^^^^^^^^^^^
        //           >
        //    ^^^^^^^^^^^^

        //      ^^^^^^^^^^^^^
        //           <
        //    ^^^^^^^^^^^^^^^

        //      ^^^^^^^^^^^^^
        //           <
        //    ^^^^^^^^^^^^^^^^^^^

        // Greater starting point

        //      ^^^^^^^^^^^^^
        //            <
        //    ^^^^^^^^^^^^^^^^^^

        //      ^^^^^^^^^^^^^
        //            <
        //            ^^^^^^^^^^

        //      ^^^^^^^^^^^^^
        //            <
        //    ^^^^^^^^^^^^^^^

        //      ^^^^^^^^^^^^^
        //           ==
        //      ^^^^^^^^^^^^^

        //      ^^^^^^^^^^^^^
        //           >
        //         ^^^^^^^^^^

        (source_file, end, other_start).cmp(&(other_source_file, other_end, start))
    }
}
