#[salsa::memoized(in crate::Jar ref)]
pub fn brew(db: &dyn crate::Db, code: Code) {}
