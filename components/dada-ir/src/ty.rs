use salsa::DebugWithDb;

#[salsa::interned(Ty in super::Jar)]
#[derive(PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum TyData {}

impl DebugWithDb<dyn crate::Db + '_> for Ty {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>, _db: &dyn crate::Db) -> std::fmt::Result {
        unreachable!()
    }
}
