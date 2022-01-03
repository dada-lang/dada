use arc_swap::ArcSwapOption;
use crossbeam::atomic::AtomicCell;
use dada_ir::diagnostic::Fallible;

use crate::{interpreter::Interpreter, moment::Moment};

use super::{invalidated::Invalidated, Permission, PermissionData};

pub(super) struct Shared {
    granted: Moment,
    canceled: Invalidated,
}

impl From<Shared> for PermissionData {
    fn from(v: Shared) -> Self {
        Self::Shared(v)
    }
}

impl Shared {
    pub(super) fn new(granted: Moment) -> Self {
        Self {
            granted,
            canceled: Default::default(),
        }
    }

    pub(super) fn cancel(&self, interpreter: &Interpreter<'_>, moment: Moment) -> Fallible<()> {
        self.canceled.invalidate(interpreter, moment)?;
        Ok(())
    }

    pub(super) fn share(
        &self,
        this: &Permission,
        interpreter: &Interpreter<'_>,
        moment: Moment,
    ) -> Fallible<Permission> {
        self.canceled.check_still_valid(interpreter, moment)?;
        Ok(this.duplicate())
    }
}
