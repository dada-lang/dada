use dada_ir::filename::Filename;

#[salsa::jar(Db)]
pub struct Jar(source_text);

pub trait Db: salsa::DbWithJar<Jar> {}
impl<T> Db for T where T: salsa::DbWithJar<Jar> {}

#[salsa::memoized(in Jar ref)]
#[allow(clippy::needless_lifetimes)]
pub fn source_text(_db: &dyn Db, _filename: Filename) -> String {
    panic!("input")
}
