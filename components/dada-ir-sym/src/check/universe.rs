use std::num::NonZeroU32;

/// A "universe" defines the set of all terms (types/permissions/etc) in a program.
/// The root universe [`Universe::ROOT`][] consists of the terms the user wrote.
/// We create other universes synthetically to create free universal variables.
/// For example, in a closure body, we are in a distinct universe, which allows us to
/// define closures that can reference a (generic) type `T` that doesn't exist in the parent universe.
///
/// Universes are ordered. `U1 < U2` means that `U2` can contain strictly more terms than `U1`.
/// `Universe::ROOT <= U` for all `U`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Universe(NonZeroU32);

impl Universe {
    /// The universe containing the things the user themselves wrote.
    /// `ROOT` <= all other universes.
    pub const ROOT: Universe = unsafe {
        // SAFETY: 0 != 1
        Universe(NonZeroU32::new_unchecked(1))
    };

    /// Create a universe one larger than the current universe.
    #[expect(dead_code)]
    pub fn next(self) -> Universe {
        Universe(self.0.checked_add(1).unwrap())
    }
}
