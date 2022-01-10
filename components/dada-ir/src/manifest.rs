use crate::filename::Filename;

#[salsa::memoized(in crate::Jar ref)]
#[allow(clippy::needless_lifetimes)]
pub fn source_text(_db: &dyn crate::Db, _filename: Filename) -> String {
    panic!("input")
}
