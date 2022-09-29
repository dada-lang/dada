use crate::code::syntax::AtomicKeyword;

/// NB: Ordering is significant. As we traverse a path, we take the
/// max of the atomic properties for the various storage modes,
/// and we want that to be atomic if any step was atomic.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Atomic {
    No,
    Yes,
}

impl From<Option<AtomicKeyword>> for Atomic {
    fn from(o: Option<AtomicKeyword>) -> Self {
        if o.is_some() {
            Atomic::Yes
        } else {
            Atomic::No
        }
    }
}

impl std::ops::BitOr for Atomic {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.max(rhs)
    }
}

impl std::ops::BitOrAssign for Atomic {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = rhs.max(*self);
    }
}

/// NB: Ordering is significant. As we traverse a path, we take the
/// max of the joint properties for the various storage modes,
/// and we want that to be atomic if any step was joint.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Joint {
    No,
    Yes,
}

impl From<Joint> for bool {
    fn from(j: Joint) -> Self {
        matches!(j, Joint::Yes)
    }
}

impl std::ops::BitOr for Joint {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.max(rhs)
    }
}

impl std::ops::BitOrAssign for Joint {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = rhs.max(*self);
    }
}

/// NB: Ordering is significant. As we traverse a path, we take the
/// max of the owned properties for the various storage modes,
/// and we want that to be atomic if any step was joint.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Leased {
    No,
    Yes,
}

impl From<Leased> for bool {
    fn from(l: Leased) -> Self {
        matches!(l, Leased::Yes)
    }
}

impl std::ops::BitOr for Leased {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.max(rhs)
    }
}

impl std::ops::BitOrAssign for Leased {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = rhs.max(*self);
    }
}
