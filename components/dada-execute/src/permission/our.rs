use dada_ir::diagnostic::Fallible;

use crate::{interpreter::Interpreter, moment::Moment};

use super::{Permission, PermissionData};

pub(super) struct Our {
    granted: Moment,
}

impl From<Our> for PermissionData {
    fn from(v: Our) -> Self {
        Self::Our(v)
    }
}

impl Our {
    pub(super) fn new(granted: Moment) -> Self {
        Self { granted }
    }

    pub(super) fn share(
        &self,
        this: &Permission,
        interpreter: &Interpreter<'_>,
        moment: Moment,
    ) -> Fallible<Permission> {
        Ok(this.duplicate())
    }
}
