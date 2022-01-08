pub struct InIrDb<'me, T: ?Sized> {
    this: &'me T,
    db: &'me dyn crate::Db,
}

impl<'me, T> InIrDb<'me, T> {
    pub fn db(&self) -> &'me dyn crate::Db {
        self.db
    }
}

impl<'me, T: ?Sized> std::ops::Deref for InIrDb<'me, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.this
    }
}

pub trait InIrDbExt {
    fn in_ir_db<'me>(&'me self, db: &'me dyn crate::Db) -> InIrDb<'me, Self>;
}

impl<T: ?Sized> InIrDbExt for T {
    fn in_ir_db<'me>(&'me self, db: &'me dyn crate::Db) -> InIrDb<'me, Self> {
        InIrDb { this: self, db }
    }
}
