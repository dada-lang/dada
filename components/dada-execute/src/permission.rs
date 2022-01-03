use std::sync::Arc;

use dada_ir::diagnostic::Fallible;

mod invalidated;
mod leased;
mod my;
mod our;
mod shared;
mod tenant;

use crate::{interpreter::Interpreter, moment::Moment};

pub struct Permission {
    data: Arc<PermissionData>,
}

impl Permission {
    fn allocate(data: impl Into<PermissionData>) -> Self {
        Self {
            data: Arc::new(data.into()),
        }
    }

    fn my(granted: Moment) -> Self {
        Self::allocate(my::My::new(granted))
    }

    fn leased(granted: Moment) -> Self {
        Self::allocate(leased::Leased::new(granted))
    }

    fn shared(granted: Moment) -> Self {
        Self::allocate(shared::Shared::new(granted))
    }

    fn our(granted: Moment) -> Self {
        Self::allocate(our::Our::new(granted))
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

    /// Given `var q = p.give`, what permission does `q` get?
    ///
    /// May also affect the permissions of `p`!
    pub fn give(&self, interpreter: &Interpreter<'_>, moment: Moment) -> Fallible<Permission> {
        self.data.give(self, interpreter, moment)
    }

    /// Given `var q = p.lease`, what permission does `q` get?
    ///
    /// May also affect the permissions of `p`!
    pub fn lease(&self, interpreter: &Interpreter<'_>, moment: Moment) -> Fallible<Permission> {
        self.data.lease(self, interpreter, moment)
    }

    /// Given `var q = p.share`, what permission does `q` get?
    ///
    /// May also affect the permissions of `p`!
    pub fn share(&self, interpreter: &Interpreter<'_>, moment: Moment) -> Fallible<Permission> {
        self.data.share(self, interpreter, moment)
    }

    /// Invoked when the lessor wishes to cancel a lease.
    ///
    /// Not possible for owned leases.
    pub fn cancel(&self, interpreter: &Interpreter<'_>, moment: Moment) -> Fallible<()> {
        self.data.cancel(interpreter, moment)
    }
}

enum PermissionData {
    My(my::My),
    Leased(leased::Leased),
    Our(our::Our),
    Shared(shared::Shared),
}

impl PermissionData {
    /// See [`Permission::give`]
    fn give(
        &self,
        this: &Permission,
        interpreter: &Interpreter<'_>,
        moment: Moment,
    ) -> Fallible<Permission> {
        match self {
            PermissionData::My(p) => p.give(interpreter, moment),

            // For exclusive, leased permissions, giving is the same as subleasing:
            PermissionData::Leased(p) => p.lease(interpreter, moment),

            // For non-exclusive permisions, giving is the same as sharing:
            PermissionData::Shared(p) => p.share(this, interpreter, moment),
            PermissionData::Our(p) => p.share(this, interpreter, moment),
        }
    }

    /// See [`Permission::lease`]
    fn lease(
        &self,
        this: &Permission,
        interpreter: &Interpreter<'_>,
        moment: Moment,
    ) -> Fallible<Permission> {
        match self {
            PermissionData::My(p) => p.lease(interpreter, moment),
            PermissionData::Leased(p) => p.lease(interpreter, moment),

            // For non-exclusive permisions, leasing is the same as sharing:
            PermissionData::Shared(p) => p.share(this, interpreter, moment),
            PermissionData::Our(p) => p.share(this, interpreter, moment),
        }
    }

    /// See [`Permission::share`]
    fn share(
        &self,
        this: &Permission,
        interpreter: &Interpreter<'_>,
        moment: Moment,
    ) -> Fallible<Permission> {
        match self {
            PermissionData::My(p) => p.share(interpreter, moment),
            PermissionData::Leased(p) => p.share(interpreter, moment),
            PermissionData::Shared(p) => p.share(this, interpreter, moment),
            PermissionData::Our(p) => p.share(this, interpreter, moment),
        }
    }

    /// See [`Permission::cancel`]
    fn cancel(&self, interpreter: &Interpreter<'_>, moment: Moment) -> Fallible<()> {
        match self {
            PermissionData::Leased(p) => p.cancel(interpreter, moment),
            PermissionData::Shared(p) => p.cancel(interpreter, moment),
            PermissionData::My(_) | PermissionData::Our(_) => {
                unreachable!("cannot cancel an owned permission")
            }
        }
    }
}
