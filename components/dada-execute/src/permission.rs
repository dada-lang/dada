use std::sync::Arc;

use arc_swap::{ArcSwap, ArcSwapOption};
use crossbeam::atomic::AtomicCell;
use dada_ir::{
    diagnostic::{self, Diagnostic, Fallible, Severity},
    error,
};

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

    pub fn my(granted: Moment) -> Self {
        Permission::allocate(My::new(granted))
    }

    fn leased(granted: Moment) -> Self {
        Permission::allocate(Leased::new(granted))
    }
}

enum PermissionData {
    My(My),
    Leased(Leased),
    Our(Our),
    Shared(Shared),
}

impl PermissionData {
    fn cancel(&self, interpreter: &Interpreter<'_>, moment: Moment) -> Fallible<()> {
        match self {
            PermissionData::My(p) => p.cancel(interpreter, moment),
            PermissionData::Leased(p) => p.cancel(interpreter, moment),
            PermissionData::Our(p) => p.cancel(interpreter, moment),
            PermissionData::Shared(p) => p.cancel(interpreter, moment),
        }
    }
}

struct My {
    granted: Moment,
    canceled: AtomicCell<Option<Moment>>,
    tenant: ArcSwapOption<PermissionData>,
}

impl From<My> for PermissionData {
    fn from(m: My) -> Self {
        Self::My(m)
    }
}

impl My {
    fn new(granted: Moment) -> Self {
        Self {
            granted,
            canceled: Default::default(),
            tenant: Default::default(),
        }
    }

    fn give(&self, interpreter: &Interpreter<'_>, moment: Moment) -> Fallible<Permission> {
        self.cancel(interpreter, moment)?;
        Ok(Permission::my(moment))
    }

    fn lease(&self, interpreter: &Interpreter<'_>, moment: Moment) -> Fallible<Permission> {
        self.check_canceled(interpreter, moment)?;
        self.cancel_tenant(interpreter, moment)?;
        let perm = Permission::leased(moment);
        self.tenant.store(Some(perm.data.clone()));
        Ok(perm)
    }

    fn share(&self, interpreter: &Interpreter<'_>, moment: Moment) -> Fallible<Permission> {
        self.check_canceled(interpreter, moment)?;
        self.cancel_tenant(interpreter, moment)?;
        let perm = Permission::leased(moment);
        self.tenant.store(Some(perm.data.clone()));
        Ok(perm)
    }

    fn cancel(&self, interpreter: &Interpreter<'_>, moment: Moment) -> Fallible<()> {
        self.check_canceled(interpreter, moment)?;
        self.cancel_tenant(interpreter, moment)?;
        self.canceled.store(Some(moment));
        Ok(())
    }

    fn cancel_tenant(&self, interpreter: &Interpreter<'_>, moment: Moment) -> Fallible<()> {
        let tenant = self.tenant.load();
        if let Some(tenant) = &*tenant {
            tenant.cancel(interpreter, moment)?;
        }
        self.tenant.store(None);
        Ok(())
    }

    fn check_canceled(&self, interpreter: &Interpreter<'_>, moment: Moment) -> Fallible<()> {
        if let Some(previous_moment) = self.canceled.load() {
            let span_now = interpreter.span(moment);
            let span_then = interpreter.span(previous_moment);
            return Err(error!(span_now, "permission already given")
                .label(span_then, "permission given here")
                .emit(interpreter.db()));
        }
        Ok(())
    }
}

struct Leased {
    granted: Moment,
    canceled: AtomicCell<Option<Moment>>,
    tenant: ArcSwapOption<PermissionData>,
}

impl From<Leased> for PermissionData {
    fn from(v: Leased) -> Self {
        Self::Leased(v)
    }
}

impl Leased {
    fn new(granted: Moment) -> Self {
        Self {
            granted,
            canceled: Default::default(),
            tenant: Default::default(),
        }
    }
}

struct Our {
    granted: Moment,
}

impl From<Our> for PermissionData {
    fn from(v: Our) -> Self {
        Self::Our(v)
    }
}

impl Our {
    fn new(granted: Moment) -> Self {
        Self { granted }
    }
}

struct Shared {
    granted: Moment,
    canceled: AtomicCell<Option<Moment>>,
    next: ArcSwapOption<PermissionData>,
}

impl From<Shared> for PermissionData {
    fn from(v: Shared) -> Self {
        Self::Shared(v)
    }
}

impl Shared {
    fn new(granted: Moment) -> Self {
        Self {
            granted,
            canceled: Default::default(),
            next: Default::default(),
        }
    }
}
