use dada_util::FromImpls;
use salsa::Update;
use serde::Serialize;

use crate::{
    ast::{AstAggregate, AstFunction},
    inputs::SourceFile,
};

#[derive(
    Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Update, FromImpls, Serialize,
)]
pub enum Anchor<'db> {
    SourceFile(SourceFile),
    Class(AstAggregate<'db>),
    Function(AstFunction<'db>),
}

impl<'db> Anchor<'db> {
    pub fn span(&self, db: &'db dyn crate::Db) -> Span<'db> {
        match self {
            Anchor::SourceFile(source_file) => Span {
                anchor: *self,
                start: Offset::ZERO,
                end: Offset::from(source_file.contents_if_ok(db).len()),
            },
            Anchor::Class(data) => data.span(db),
            Anchor::Function(data) => data.span(db),
        }
    }

    /// Compute the absolute span of this anchor's contents.
    pub fn absolute_span_of_contents(&self, db: &'db dyn crate::Db) -> AbsoluteSpan {
        match self {
            Anchor::SourceFile(source_file) => source_file.absolute_span(db),

            // For most anchors, we have to skip past the `{}` or `()` in the delimiters by invoking `narrow`.
            Anchor::Class(data) => data.span(db).absolute_span(db).narrow(),
            Anchor::Function(data) => data.span(db).absolute_span(db).narrow(),
        }
    }

    pub fn source_file(&self, db: &dyn crate::Db) -> SourceFile {
        match self {
            Anchor::SourceFile(source_file) => *source_file,
            Anchor::Class(ast_class_item) => ast_class_item.name_span(db).source_file(db),
            Anchor::Function(ast_function) => ast_function.name(db).span.source_file(db),
        }
    }
}

/// A span within the input.
///
/// The offsets are stored relative to the start of the **anchor**,
/// which is some item (e.g., a class, function, etc). The use of relative offsets avoids
/// incremental churn if lines or content is added before/after the definition.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Update)]
pub struct Span<'db> {
    pub anchor: Anchor<'db>,
    pub start: Offset,
    pub end: Offset,
}

impl serde::Serialize for Span<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        salsa::with_attached_database(|db| {
            let db: &dyn crate::Db = db.as_view();
            serde::Serialize::serialize(&self.absolute_span(db), serializer)
        })
        .unwrap_or_else(|| panic!("cannot serialize without attached database"))
    }
}

impl std::fmt::Debug for Span<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Span")
            .field("start", &self.start)
            .field("end", &self.end)
            .field("anchor", &"...")
            .finish()
    }
}

/// An absolute span within the input. The offsets are stored as absolute offsets
/// within a given source file. These are used for diagnostics or outputs but not
/// internally during compilation.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize)]
pub struct AbsoluteSpan {
    pub source_file: SourceFile,
    pub start: AbsoluteOffset,
    pub end: AbsoluteOffset,
}

impl AbsoluteSpan {
    /// Skip one character at the start/end of the span.
    /// Used to skip past delimiters when computing absolute spans.
    pub fn narrow(mut self) -> Self {
        self.start = self.start + Offset::ONE;
        self.end = self.end - Offset::ONE;
        assert!(self.start <= self.end);
        self
    }

    /// Convert into a span anchored at the source file.
    pub fn into_span(self, _db: &dyn crate::Db) -> Span<'_> {
        Span {
            anchor: self.source_file.into(),
            start: Offset::from(self.start),
            end: Offset::from(self.end),
        }
    }
}

impl<'db> Span<'db> {
    pub fn to(self, db: &'db dyn crate::Db, end: impl IntoOptionSpan<'db>) -> Span<'db> {
        match end.into_opt_span() {
            Some(end) => {
                if self.anchor == end.anchor {
                    Span {
                        anchor: self.anchor,
                        start: self.start,
                        end: end.end,
                    }
                } else {
                    // this invariant can fail when errors occur etc.
                    // for now just convert to absolute spans, though we
                    // could probably be more precise.
                    self.absolute_span(db)
                        .into_span(db)
                        .to(db, end.absolute_span(db).into_span(db))
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
        let anchor_span = self.anchor.absolute_span_of_contents(db);
        AbsoluteSpan {
            source_file: anchor_span.source_file,
            start: anchor_span.start + self.start,
            end: anchor_span.start + self.end,
        }
    }

    pub fn source_file(&self, db: &dyn crate::Db) -> SourceFile {
        self.anchor.source_file(db)
    }
}

impl<'db> Spanned<'db> for Span<'db> {
    fn span(&self, _db: &'db dyn crate::Db) -> Span<'db> {
        *self
    }
}

/// Implemented by all things that have a span (and span itself).
///
/// For AST nodes, yields the entire encompassing span.
///
/// For symbols, yields a span intended for use in error reporting.
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

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize)]
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

impl From<AbsoluteOffset> for Offset {
    fn from(offset: AbsoluteOffset) -> Self {
        Offset(offset.0)
    }
}

impl Offset {
    pub const ZERO: Offset = Offset(0);
    pub const ONE: Offset = Offset(1);

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

impl std::ops::Sub<Offset> for Offset {
    type Output = Offset;

    fn sub(self, rhs: Offset) -> Self::Output {
        Offset(self.0.checked_sub(rhs.0).unwrap())
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize)]
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

impl std::ops::Sub<Offset> for AbsoluteOffset {
    type Output = AbsoluteOffset;

    fn sub(self, rhs: Offset) -> Self::Output {
        AbsoluteOffset(self.0.checked_sub(rhs.0).unwrap())
    }
}

impl std::ops::Sub<AbsoluteOffset> for AbsoluteOffset {
    type Output = u32;

    fn sub(self, rhs: AbsoluteOffset) -> Self::Output {
        self.0.checked_sub(rhs.0).unwrap()
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

/// A zero-based line number
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize)]
pub struct ZeroLine(u32);

/// A one-based line number
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize)]
pub struct OneLine(u32);

/// A zero-based column number
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize)]
pub struct ZeroColumn(u32);

/// A one-based column number
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize)]
pub struct OneColumn(u32);

macro_rules! methods {
    ($t:ident) => {
        impl $t {
            pub fn as_u32(self) -> u32 {
                self.0
            }

            pub fn as_usize(self) -> usize {
                self.0 as usize
            }
        }

        impl From<usize> for $t {
            fn from(offset: usize) -> Self {
                assert!(offset < u32::MAX as usize);
                $t(offset as u32)
            }
        }

        impl From<u32> for $t {
            fn from(offset: u32) -> Self {
                $t(offset)
            }
        }

        impl std::ops::Add<$t> for $t {
            type Output = $t;

            fn add(self, rhs: $t) -> Self::Output {
                $t(self.0.checked_add(rhs.0).unwrap())
            }
        }

        impl std::ops::Sub<$t> for $t {
            type Output = $t;

            fn sub(self, rhs: $t) -> Self::Output {
                $t(self.0.checked_sub(rhs.0).unwrap())
            }
        }

        impl std::ops::Add<u32> for $t {
            type Output = $t;

            fn add(self, rhs: u32) -> Self::Output {
                $t(self.0.checked_add(rhs).unwrap())
            }
        }

        impl std::ops::Sub<u32> for $t {
            type Output = $t;

            fn sub(self, rhs: u32) -> Self::Output {
                $t(self.0.checked_sub(rhs).unwrap())
            }
        }
    };
}

methods!(ZeroColumn);
methods!(ZeroLine);
methods!(OneColumn);
methods!(OneLine);
