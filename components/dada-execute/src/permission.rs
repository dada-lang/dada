use std::sync::Arc;

use dada_ir::diagnostic::Fallible;

mod invalidated;
mod leased;
mod my;
mod our;
mod shared;
mod tenant;

use crate::interpreter::Interpreter;

#[derive(Debug)]
pub(crate) struct Permission {
    data: Arc<PermissionData>,
}

impl Permission {
    fn allocate(data: impl Into<PermissionData>) -> Self {
        Self {
            data: Arc::new(data.into()),
        }
    }

    fn my(interpreter: &Interpreter<'_>) -> Self {
        Self::allocate(my::My::new(interpreter))
    }

    fn leased(interpreter: &Interpreter<'_>) -> Self {
        Self::allocate(leased::Leased::new(interpreter))
    }

    fn shared(interpreter: &Interpreter<'_>) -> Self {
        Self::allocate(shared::Shared::new(interpreter))
    }

    fn our(interpreter: &Interpreter<'_>) -> Self {
        Self::allocate(our::Our::new(interpreter))
    }

    /// Duplicates thie permision. Must be a non-affine permission.
    fn duplicate(&self) -> Self {
        assert!(matches!(
            &*self.data,
            PermissionData::Our(_) | PermissionData::Shared(_)
        ));

        Permission {
            data: self.data.clone(),
        }
    }

    /// Checks that this permission permits reading of a field.
    pub(crate) fn check_read(&self, interpreter: &Interpreter<'_>) -> Fallible<()> {
        self.data.check_read(interpreter)
    }

    /// Given `var q = p.give`, what permission does `q` get?
    ///
    /// May also affect the permissions of `p`!
    pub(crate) fn give(&self, interpreter: &Interpreter<'_>) -> Fallible<Permission> {
        self.data.give(self, interpreter)
    }

    /// Given `var q = p.lease`, what permission does `q` get?
    ///
    /// May also affect the permissions of `p`!
    pub(crate) fn lease(&self, interpreter: &Interpreter<'_>) -> Fallible<Permission> {
        self.data.lease(self, interpreter)
    }

    /// Given `var q = p.share`, what permission does `q` get?
    ///
    /// May also affect the permissions of `p`!
    pub(crate) fn share(&self, interpreter: &Interpreter<'_>) -> Fallible<Permission> {
        self.data.share(self, interpreter)
    }

    /// Invoked when the lessor wishes to cancel a lease.
    ///
    /// Not possible for owned leases.
    pub(crate) fn cancel(&self, interpreter: &Interpreter<'_>) -> Fallible<()> {
        self.data.cancel(interpreter)
    }
}

#[derive(Debug)]
enum PermissionData {
    My(my::My),
    Leased(leased::Leased),
    Our(our::Our),
    Shared(shared::Shared),
}

impl PermissionData {
    /// True if this is an *exclusive* permision, meaning that while it is valid, no access cannot occur through an alias.
    ///
    /// The opposite of an exclusive permission is a *shared* permision, which permit reads throug aliases.
    fn exclusive(&self) -> bool {
        match self {
            PermissionData::My(_) | PermissionData::Leased(_) => true,
            PermissionData::Our(_) | PermissionData::Shared(_) => false,
        }
    }

    /// See [`Permission::give`]
    fn give(&self, this: &Permission, interpreter: &Interpreter<'_>) -> Fallible<Permission> {
        match self {
            PermissionData::My(p) => p.give(interpreter),

            // For exclusive, leased permissions, giving is the same as subleasing:
            PermissionData::Leased(p) => p.lease(interpreter),

            // For non-exclusive permisions, giving is the same as sharing:
            PermissionData::Shared(p) => p.share(this, interpreter),
            PermissionData::Our(p) => p.share(this, interpreter),
        }
    }

    /// See [`Permission::lease`]
    fn lease(&self, this: &Permission, interpreter: &Interpreter<'_>) -> Fallible<Permission> {
        match self {
            PermissionData::My(p) => p.lease(interpreter),
            PermissionData::Leased(p) => p.lease(interpreter),

            // For non-exclusive permisions, leasing is the same as sharing:
            PermissionData::Shared(p) => p.share(this, interpreter),
            PermissionData::Our(p) => p.share(this, interpreter),
        }
    }

    /// See [`Permission::share`]
    fn share(&self, this: &Permission, interpreter: &Interpreter<'_>) -> Fallible<Permission> {
        match self {
            PermissionData::My(p) => p.share(interpreter),
            PermissionData::Leased(p) => p.share(interpreter),
            PermissionData::Shared(p) => p.share(this, interpreter),
            PermissionData::Our(p) => p.share(this, interpreter),
        }
    }

    /// See [`Permission::cancel`]
    fn cancel(&self, interpreter: &Interpreter<'_>) -> Fallible<()> {
        match self {
            PermissionData::Leased(p) => p.cancel(interpreter),
            PermissionData::Shared(p) => p.cancel(interpreter),
            PermissionData::My(_) | PermissionData::Our(_) => {
                unreachable!("cannot cancel an owned permission")
            }
        }
    }

    /// See [`Permission::check_read`]
    fn check_read(&self, interpreter: &Interpreter<'_>) -> Fallible<()> {
        match self {
            PermissionData::My(p) => p.check_read(interpreter),
            PermissionData::Leased(p) => p.check_read(interpreter),
            PermissionData::Shared(p) => p.check_read(interpreter),
            PermissionData::Our(p) => p.check_read(interpreter),
        }
    }

    /// See [`Permission::check_write`]
    fn check_write(&self, interpreter: &Interpreter<'_>) -> Fallible<()> {
        match self {
            PermissionData::My(p) => p.check_write(interpreter),
            PermissionData::Leased(p) => p.check_write(interpreter),
            PermissionData::Shared(p) => p.check_write(interpreter),
            PermissionData::Our(p) => p.check_write(interpreter),
        }
    }
}
